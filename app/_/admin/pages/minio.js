(function () {
    const baseUrl = window.location.origin.replace('https://', 'http://'); // 将https改为http
    const targetUrl = baseUrl + ":9001"; // 目标地址为当前域名加端口
    const response = {
        data: {
            type: "page",
            title: "Minio管理后台",
            body: [
            {
                type: "qr-code",
                codeSize: 128,
                value: targetUrl
            },{
                type: "link",
				icon: "fa-solid fa-link",
                href: targetUrl,
                body: "Minio 管理后台"
            }]
        },
        status: 0
    };
    // 检查是否已经打开了目标页面
    if (window.minioAdminWindow) {
        // 如果已经存在窗口引用，则切换到该窗口
        window.minioAdminWindow.focus();
    } else {
        // 如果不存在窗口引用，则在新标签页中打开
        window.minioAdminWindow = window.open(targetUrl, "_blank");
    }
    // 调用 JSONP 回调
    window.jsonpCallback && window.jsonpCallback(response);
})();