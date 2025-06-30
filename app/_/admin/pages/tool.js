// (function() {
// 	const response = {
// 		data: {
// 			type: "page",
// 			// title: "查询工具",
// 			body: "收录 排名 查询工具 开发中..."
// 		},
// 		status: 0
// 	}

// 	window.jsonpCallback && window.jsonpCallback(response);
// })();

(function () {
    const response = {
        data: {
            "type": "page",
            "body": {
                "type": "iframe",
                "src": "https://tools.datapipe.top/"
            }
        },
        status: 0
    }

    window.jsonpCallback && window.jsonpCallback(response);
})();
