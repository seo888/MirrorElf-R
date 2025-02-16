(function () {
	const baseUrl = window.location.origin;  // 获取当前页面的域名

	const response = {
		data: {
			"type": "iframe",
			"className": "b-a",
			"src": baseUrl + ":9001",  // 使用当前页面的域名加端口
			"style": {
				"maxHeight": "960px"   // 设置最小高度，防止内容太少时高度太小
			}
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();

