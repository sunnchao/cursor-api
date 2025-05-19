import { connect } from "cloudflare:sockets";

// Global configuration including the authentication token, default destination URL, and debug mode flag
const CONFIG = {
  AUTH_TOKEN: "image",
  DEFAULT_DST_URL: "https://example.com/",
  DEBUG_MODE: false,
};

// Update global configuration from environment variables (prioritizing environment values)
function updateConfigFromEnv(env) {
  if (!env) return;
  for (const key of Object.keys(CONFIG)) {
    if (key in env) {
      if (typeof CONFIG[key] === 'boolean') {
        CONFIG[key] = env[key] === 'true';
      } else {
        CONFIG[key] = env[key];
      }
    }
  }
}

// Define text encoder and decoder for converting between strings and byte arrays
const encoder = new TextEncoder();
const decoder = new TextDecoder();

// Filter out HTTP headers that should not be forwarded (ignore headers: host, x-forwarded-proto, x-real-ip, cf-*)
const HEADER_FILTER_RE = /^(host|x-co|x-forwarded-proto|x-real-ip|cf-)/i;

// Define the debug log output function based on the debug mode setting
const log = CONFIG.DEBUG_MODE
  ? (message, data = "") => console.log(`[DEBUG] ${message}`, data)
  : () => { };

// Concatenate multiple Uint8Arrays into a single new Uint8Array
function concatUint8Arrays(...arrays) {
  const total = arrays.reduce((sum, arr) => sum + arr.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }
  return result;
}

// Parse HTTP response headers, returning the status code, status text, headers, and the header section's end position
function parseHttpHeaders(buff) {
  const text = decoder.decode(buff);
  // Look for the end of HTTP headers indicated by "\r\n\r\n"
  const headerEnd = text.indexOf("\r\n\r\n");
  if (headerEnd === -1) return null;
  const headerSection = text.slice(0, headerEnd).split("\r\n");
  const statusLine = headerSection[0];
  // Match the HTTP status line, e.g., "HTTP/1.1 200 OK"
  const statusMatch = statusLine.match(/HTTP\/1\.[01] (\d+) (.*)/);
  if (!statusMatch) throw new Error(`Invalid status line: ${statusLine}`);
  const headers = new Headers();
  // Parse the response headers
  for (let i = 1; i < headerSection.length; i++) {
    const line = headerSection[i];
    const idx = line.indexOf(": ");
    if (idx !== -1) {
      headers.append(line.slice(0, idx), line.slice(idx + 2));
    }
  }
  return { status: Number(statusMatch[1]), statusText: statusMatch[2], headers, headerEnd };
}

// Read data from the reader until a double CRLF (indicating the end of HTTP headers) is encountered
async function readUntilDoubleCRLF(reader) {
  let respText = "";
  while (true) {
    const { value, done } = await reader.read();
    if (value) {
      respText += decoder.decode(value, { stream: true });
      if (respText.includes("\r\n\r\n")) break;
    }
    if (done) break;
  }
  return respText;
}

// Async generator: read chunked HTTP response data chunks and yield each chunk sequentially
async function* readChunks(reader, buff = new Uint8Array()) {
  while (true) {
    // Look for the position of the CRLF separator in the existing buffer
    let pos = -1;
    for (let i = 0; i < buff.length - 1; i++) {
      if (buff[i] === 13 && buff[i + 1] === 10) {
        pos = i;
        break;
      }
    }
    // If not found, continue reading more data to fill the buffer
    if (pos === -1) {
      const { value, done } = await reader.read();
      if (done) break;
      buff = concatUint8Arrays(buff, value);
      continue;
    }
    // Parse the chunk size (in hexadecimal format)
    const size = parseInt(decoder.decode(buff.slice(0, pos)), 16);
    log("Read chunk size", size.toString());
    // A size of 0 indicates the end of chunks
    if (!size) break;
    // Remove the parsed size part and the following CRLF from the buffer
    buff = buff.slice(pos + 2);
    // Ensure the buffer contains the complete chunk (including the trailing CRLF)
    while (buff.length < size + 2) {
      const { value, done } = await reader.read();
      if (done) throw new Error("Unexpected EOF in chunked encoding");
      buff = concatUint8Arrays(buff, value);
    }
    // Yield the chunk data (excluding the trailing CRLF)
    yield buff.slice(0, size);
    buff = buff.slice(size + 2);
  }
}

// Parse the complete HTTP response, handling the response body data based on transfer mode (chunked or fixed-length)
async function parseResponse(reader) {
  let buff = new Uint8Array();
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (value) {
        buff = concatUint8Arrays(buff, value);
        const parsed = parseHttpHeaders(buff);
        if (parsed) {
          const { status, statusText, headers, headerEnd } = parsed;
          log(`收到响应: ${status} ${statusText}`);
          for (const [k, v] of headers.entries()) {
            log(`响应头部: ${k} = ${v}`);
          }

          const isChunked = headers.get("transfer-encoding")?.includes("chunked");
          const contentLength = parseInt(headers.get("content-length") || "0", 10);
          const data = buff.slice(headerEnd + 4);

          // Distribute the response body data via a ReadableStream
          return new Response(
            new ReadableStream({
              async start(ctrl) {
                try {
                  if (isChunked) {
                    log("使用分块传输模式");
                    // Chunked transfer mode: read and enqueue each chunk sequentially
                    for await (const chunk of readChunks(reader, data)) {
                      log(`转发分块: ${chunk.length} 字节`);
                      ctrl.enqueue(chunk);
                    }
                  } else {
                    log("使用固定长度模式", JSON.stringify({ contentLength }));
                    let received = data.length;
                    if (data.length) {
                      log(`转发初始数据: ${data.length} 字节`);
                      ctrl.enqueue(data);
                    }
                    // Fixed-length mode: read the specified number of bytes based on content-length
                    try {
                      while (received < contentLength) {
                        const { value, done } = await reader.read();
                        if (done) {
                          log(`提前结束: 已接收 ${received} / ${contentLength} 字节`);
                          break;
                        }
                        received += value.length;
                        log(`转发数据: ${value.length} 字节`);
                        ctrl.enqueue(value);
                      }
                    } catch (err) {
                      log(`读取响应体错误: ${err.message}`);
                      throw err;
                    }
                  }
                  ctrl.close();
                } catch (err) {
                  log("响应流错误", err);
                  ctrl.error(err);
                }
              },
              cancel() {
                log("响应流被取消");
              }
            }),
            { status, statusText, headers }
          );
        }
      }
      if (done) {
        log("读取结束，但未找到完整响应头");
        break;
      }
    }
  } catch (err) {
    log("解析响应时发生错误", err);
    throw err;
  }
  throw new Error("无法解析响应头");
}

// Generate a random Sec-WebSocket-Key required for the WebSocket handshake
function generateWebSocketKey() {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return btoa(String.fromCharCode(...bytes));
}

// Pack a text message into a WebSocket frame (currently supports only text frames with payloads not too large)
function packTextFrame(payload) {
  const FIN_AND_OP = 0x81; // FIN flag and text frame opcode
  const maskBit = 0x80; // Mask bit (must be set to 1 for client-sent messages)
  const len = payload.length;
  let header;
  if (len < 126) {
    header = new Uint8Array(2);
    header[0] = FIN_AND_OP;
    header[1] = maskBit | len;
  } else if (len < 65536) {
    header = new Uint8Array(4);
    header[0] = FIN_AND_OP;
    header[1] = maskBit | 126;
    header[2] = (len >> 8) & 0xff;
    header[3] = len & 0xff;
  } else {
    throw new Error("Payload too large");
  }
  // Generate a 4-byte random mask
  const mask = new Uint8Array(4);
  crypto.getRandomValues(mask);
  const maskedPayload = new Uint8Array(len);
  // Apply the mask to the payload
  for (let i = 0; i < len; i++) {
    maskedPayload[i] = payload[i] ^ mask[i % 4];
  }
  // Concatenate the frame header, mask, and masked payload
  return concatUint8Arrays(header, mask, maskedPayload);
}

// Class for parsing and reassembling WebSocket frames, supporting fragmented messages
class SocketFramesReader {
  constructor(reader) {
    this.reader = reader;
    this.buffer = new Uint8Array();
    this.fragmentedPayload = null;
    this.fragmentedOpcode = null;
  }
  // Ensure that the buffer has enough bytes for parsing
  async ensureBuffer(length) {
    while (this.buffer.length < length) {
      const { value, done } = await this.reader.read();
      if (done) return false;
      this.buffer = concatUint8Arrays(this.buffer, value);
    }
    return true;
  }
  // Parse the next WebSocket frame and handle fragmentation (opcode 0 indicates continuation)
  async nextFrame() {
    while (true) {
      if (!(await this.ensureBuffer(2))) return null;
      const first = this.buffer[0],
        second = this.buffer[1],
        fin = (first >> 7) & 1,
        opcode = first & 0x0f,
        isMasked = (second >> 7) & 1;
      let payloadLen = second & 0x7f,
        offset = 2;
      // If payload length is 126, parse the next two bytes for the actual length
      if (payloadLen === 126) {
        if (!(await this.ensureBuffer(offset + 2))) return null;
        payloadLen = (this.buffer[offset] << 8) | this.buffer[offset + 1];
        offset += 2;
      } else if (payloadLen === 127) {
        throw new Error("127 length mode is not supported");
      }
      let mask;
      if (isMasked) {
        if (!(await this.ensureBuffer(offset + 4))) return null;
        mask = this.buffer.slice(offset, offset + 4);
        offset += 4;
      }
      if (!(await this.ensureBuffer(offset + payloadLen))) return null;
      let payload = this.buffer.slice(offset, offset + payloadLen);
      if (isMasked && mask) {
        for (let i = 0; i < payload.length; i++) {
          payload[i] ^= mask[i % 4];
        }
      }
      // Remove the processed bytes from the buffer
      this.buffer = this.buffer.slice(offset + payloadLen);
      // Opcode 0 indicates a continuation frame: concatenate the fragmented data
      if (opcode === 0) {
        if (this.fragmentedPayload === null)
          throw new Error("Received continuation frame without initiation");
        this.fragmentedPayload = concatUint8Arrays(this.fragmentedPayload, payload);
        if (fin) {
          const completePayload = this.fragmentedPayload;
          const completeOpcode = this.fragmentedOpcode;
          this.fragmentedPayload = this.fragmentedOpcode = null;
          return { fin: true, opcode: completeOpcode, payload: completePayload };
        }
      } else {
        // If there is fragmented data but the current frame is not a continuation, reset the fragmentation state
        if (!fin) {
          this.fragmentedPayload = payload;
          this.fragmentedOpcode = opcode;
          continue;
        } else {
          if (this.fragmentedPayload) {
            this.fragmentedPayload = this.fragmentedOpcode = null;
          }
          return { fin, opcode, payload };
        }
      }
    }
  }
}

// Forward HTTP requests or WebSocket handshake and data based on the request type
async function nativeFetch(req, dstUrl) {
  // Clean up the headers by removing those that match the filter criteria
  const cleanedHeaders = new Headers();
  for (const [k, v] of req.headers) {
    if (!HEADER_FILTER_RE.test(k)) {
      // 确保 Connection 头部值为小写
      const value = k.toLowerCase() === "connection" ? v.toLowerCase() : v;
      cleanedHeaders.set(k, value);
      log(`转发头部: ${k}: ${value}`);
    }
  }

  // Check if the request is a WebSocket request
  const upgradeHeader = req.headers.get("Upgrade")?.toLowerCase();
  const isWebSocket = upgradeHeader === "websocket";
  const targetUrl = new URL(dstUrl);

  log(`目标URL: ${dstUrl}, 方法: ${req.method}`);

  if (isWebSocket) {
    // If the target URL does not support the WebSocket protocol, return an error response
    if (!/^wss?:\/\//i.test(dstUrl)) {
      return new Response("Target does not support WebSocket", { status: 400 });
    }
    const isSecure = targetUrl.protocol === "wss:";
    const port = targetUrl.port || (isSecure ? 443 : 80);
    // Establish a raw socket connection to the target server
    let socket;
    try {
      socket = await connect(
        { hostname: targetUrl.hostname, port: Number(port) },
        { secureTransport: isSecure ? "on" : "off", allowHalfOpen: false }
      );
    } catch (err) {
      log(`连接错误: ${err.message}`);
      throw err;
    }

    // Generate the key required for the WebSocket handshake
    const key = generateWebSocketKey();

    // Construct the HTTP headers required for the handshake
    cleanedHeaders.set('Host', targetUrl.hostname);
    cleanedHeaders.set('Connection', 'Upgrade');
    cleanedHeaders.set('Upgrade', 'websocket');
    cleanedHeaders.set('Sec-WebSocket-Version', '13');
    cleanedHeaders.set('Sec-WebSocket-Key', key);

    // Assemble the HTTP request data for the WebSocket handshake
    const handshakeReq =
      `GET ${targetUrl.pathname}${targetUrl.search} HTTP/1.1\r\n` +
      Array.from(cleanedHeaders.entries())
        .map(([k, v]) => `${k}: ${v}`)
        .join('\r\n') +
      '\r\n\r\n';

    log("Sending WebSocket handshake request", handshakeReq);
    const writer = socket.writable.getWriter();
    await writer.write(encoder.encode(handshakeReq));

    const reader = socket.readable.getReader();
    const handshakeResp = await readUntilDoubleCRLF(reader);
    log("Received handshake response", handshakeResp);
    // Verify that the handshake response indicates a 101 Switching Protocols status
    if (
      !handshakeResp.includes("101") ||
      !handshakeResp.includes("Switching Protocols")
    ) {
      throw new Error("WebSocket handshake failed: " + handshakeResp);
    }

    // Create an internal WebSocketPair
    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);
    client.accept();
    // Establish bidirectional frame relaying between the client and the remote socket
    relayWebSocketFrames(client, socket, writer, reader);
    return new Response(null, {
      status: 101,
      webSocket: server,
      headers: {
        "Access-Control-Allow-Origin": "*"
      }
    });
  } else {
    // For standard HTTP requests: set required headers (such as Host)
    cleanedHeaders.set("Host", targetUrl.hostname);

    const port = targetUrl.protocol === "https:" ? 443 : 80;
    let socket;
    try {
      socket = await connect(
        { hostname: targetUrl.hostname, port },
        { secureTransport: targetUrl.protocol === "https:" ? "on" : "off", allowHalfOpen: false }
      );
    } catch (err) {
      log(`连接错误: ${err.message}`);
      throw err;
    }

    const writer = socket.writable.getWriter();
    // Construct the request line and headers
    const requestLine =
      `${req.method} ${targetUrl.pathname}${targetUrl.search} HTTP/1.1\r\n` +
      Array.from(cleanedHeaders.entries())
        .map(([k, v]) => `${k}: ${v}`)
        .join("\r\n") +
      "\r\n\r\n";
    log("发送请求", requestLine);

    try {
      await writer.write(encoder.encode(requestLine));

      // If there is a request body, forward it to the target server
      if (req.body) {
        log("转发请求体");
        const reader = req.body.getReader();
        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            await writer.write(value);
          }
        } finally {
          reader.releaseLock();
        }
      }

      // Parse and return the target server's response
      const reader = socket.readable.getReader();
      try {
        return await parseResponse(reader);
      } catch (err) {
        log(`解析响应错误: ${err.message}`);
        throw err;
      }
    } catch (err) {
      log(`请求处理错误: ${err.message}`);
      throw err;
    } finally {
      try {
        writer.releaseLock();
        socket.close();
      } catch (err) {
        log(`关闭连接错误: ${err.message}`);
      }
    }
  }
}

// Relay WebSocket frames bidirectionally between the client and the remote socket
function relayWebSocketFrames(ws, socket, writer, reader) {
  // Listen for messages from the client, package them into frames, and send them to the remote socket
  ws.addEventListener("message", async (event) => {
    let payload;
    if (typeof event.data === "string") {
      payload = encoder.encode(event.data);
    } else if (event.data instanceof ArrayBuffer) {
      payload = new Uint8Array(event.data);
    } else {
      payload = event.data;
    }
    const frame = packTextFrame(payload);
    try {
      await writer.write(frame);
    } catch (e) {
      log("Remote write error", e);
    }
  });

  // Asynchronously relay WebSocket frames received from the remote to the client
  (async function relayFrames() {
    const frameReader = new SocketFramesReader(reader);
    try {
      while (true) {
        const frame = await frameReader.nextFrame();
        if (!frame) break;
        // Process the data frame based on its opcode
        switch (frame.opcode) {
          case 1: // Text frame
          case 2: // Binary frame
            ws.send(frame.payload);
            break;
          case 8: // Close frame
            log("Received Close frame, closing WebSocket");
            ws.close(1000);
            return;
          default:
            log(`Received unknown frame type, Opcode: ${frame.opcode}`);
        }
      }
    } catch (e) {
      log("Error reading remote frame", e);
    } finally {
      ws.close();
      writer.releaseLock();
      socket.close();
    }
  })();

  // When the client WebSocket closes, also close the remote socket connection
  ws.addEventListener("close", () => socket.close());
}

// Entry point for handling requests: update configuration, parse target URL, and forward the request
async function handleRequest(req, env) {
  updateConfigFromEnv(env);

  // 处理 CORS 预检请求
  if (req.method === "OPTIONS") {
    return new Response(null, {
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, Authorization, x-co",
        "Access-Control-Max-Age": "86400"
      }
    });
  }

  // 创建快捷的错误响应函数，同时添加 CORS 头
  const errorResponse = (status, message) =>
    new Response(message, {
      status,
      headers: {
        "Access-Control-Allow-Origin": "*"
      }
    });

  // 检查必要的 x-co 头部
  const hostHeader = req.headers.get("x-co");
  if (!hostHeader) {
    return errorResponse(400, "缺少头部");
  }

  // 检查主机白名单
  const allowedHosts = ["api2.cursor.sh", "www.cursor.com"];
  if (!allowedHosts.includes(hostHeader)) {
    return errorResponse(403, "主机被拒绝");
  }

  // 检查路径白名单
  const url = new URL(req.url);
  const allowedPaths = [
    "/aiserver.v1.AiService/StreamChat",
    "/aiserver.v1.AiService/StreamChatWeb",
    "/auth/full_stripe_profile",
    "/api/usage",
    "/api/auth/me"
  ];

  if (!allowedPaths.includes(url.pathname)) {
    return errorResponse(404, "路径无效");
  }

  // 构建目标 URL
  const dstUrl = `https://${hostHeader}${url.pathname}${url.search}`;
  log("目标 URL", dstUrl);

  try {
    // 调用 nativeFetch 转发请求
    const response = await nativeFetch(req, dstUrl);

    // 获取原始响应的头部并添加 CORS 头
    const newHeaders = new Headers(response.headers);
    newHeaders.set("Access-Control-Allow-Origin", "*");

    // 返回带有新头部的响应
    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: newHeaders
    });
  } catch (error) {
    log("请求处理错误", error);
    return errorResponse(500, "服务器错误");
  }
}

// Export the fetch event handler for Cloudflare Workers and related environments
export default { fetch: handleRequest };
export const onRequest = (ctx) => handleRequest(ctx.request, ctx.env);