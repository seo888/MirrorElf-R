// (function () {
// 	const baseUrl = window.location.origin;  // 获取当前页面的域名

// 	const response = {
// 		data: {
// 			"type": "iframe",
// 			"className": "b-a",
// 			"src": baseUrl + ":9443",  // 使用当前页面的域名加端口
// 			"style": {
// 				"maxHeight": "960px"   // 设置最小高度，防止内容太少时高度太小
// 			}
// 		},
// 		status: 0
// 	}

// 	window.jsonpCallback && window.jsonpCallback(response);
// })();

(function () {
    const baseUrl = window.location.origin; // 获取当前页面的域名
    const targetUrl = baseUrl + ":9443"; // 目标地址为当前域名加端口
    const response = {
        data: {
            type: "page",
            title: "FireWall 管理后台",
            body: [
            {
                type: "qr-code",
                codeSize: 128,
                value: targetUrl
            },{
                type: "link",
				icon: "fa-solid fa-link",
                href: targetUrl,
                body: "FireWall 管理后台"
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
