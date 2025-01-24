// 定义允许的主机和路径
const ALLOWED_HOSTS = ["api2.cursor.sh", "www.cursor.com"];
const ALLOWED_PATHS = [
  "/aiserver.v1.AiService/StreamChat",
  "/auth/full_stripe_profile",
  "/api/usage",
  "/api/auth/me"
];

// 创建统一的响应处理函数
const createResponse = (status: number, message: string) => 
  new Response(message, {
    status,
    headers: { "Access-Control-Allow-Origin": "*" }
  });

// 主处理函数
Deno.serve(async (request: Request) => {
  // 验证目标主机
  const targetHost = request.headers.get("x-co");
  if (!targetHost) return createResponse(400, "Missing header");
  if (!ALLOWED_HOSTS.includes(targetHost)) return createResponse(403, "Host denied");

  // 验证请求路径
  const url = new URL(request.url);
  if (!ALLOWED_PATHS.includes(url.pathname)) return createResponse(404, "Path invalid");

  // 处理请求头
  const headers = new Headers(request.headers);
  headers.delete("x-co");
  headers.set("Host", targetHost);

  try {
    // 转发请求
    const response = await fetch(
      `https://${targetHost}${url.pathname}${url.search}`,
      {
        method: request.method,
        headers,
        body: request.body
      }
    );

    // 处理响应头
    const responseHeaders = new Headers(response.headers);
    responseHeaders.set("Access-Control-Allow-Origin", "*");

    return new Response(response.body, {
      status: response.status,
      headers: responseHeaders
    });
  } catch (error) {
    return createResponse(500, "Server error");
  }
});
