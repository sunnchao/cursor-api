addEventListener('fetch', e => {
  e.respondWith(handleRequest(e.request));
});

async function handleRequest(request) {
  try {
    // 获取目标主机
    const targetHost = request.headers.get("x-co");
    
    // 允许的主机和路径列表
    const allowedHosts = ["api2.cursor.sh", "www.cursor.com"];
    const allowedPaths = [
      "/aiserver.v1.AiService/StreamChat",
      "/aiserver.v1.AiService/StreamChatWeb",
      "/auth/full_stripe_profile",
      "/api/usage",
      "/api/auth/me"
    ];

    const url = new URL(request.url);

    // 验证请求
    if (!targetHost || !allowedHosts.includes(targetHost) || !allowedPaths.includes(url.pathname)) {
      return new Response(null, { status: 403 });
    }

    // 处理请求头
    const headers = new Headers(request.headers);
    headers.delete("x-co");
    headers.set("Host", targetHost);

    // 转发请求
    const response = await fetch(
      `https://${targetHost}${url.pathname}${url.search}`,
      {
        method: request.method,
        headers: headers,
        body: request.body
      }
    );

    // 处理响应
    const responseHeaders = new Headers(response.headers);
    responseHeaders.set("Access-Control-Allow-Origin", "*");

    return new Response(response.body, {
      status: response.status,
      headers: responseHeaders
    });

  } catch (error) {
    // 错误处理
    console.error('Request failed:', error);
    return new Response("Internal Server Error", {
      status: 500,
      headers: { "Content-Type": "text/plain" }
    });
  }
}
