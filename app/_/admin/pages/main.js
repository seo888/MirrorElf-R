(function () {
	const response = {
		data: {
			type: "page",
			title: "仪表盘",
			body: [{
				"type": "panel",
				"title": "网站实时日志",
				"body": {
					"type": "log",
					"height": 300,
					// "rowHeight": 22,
					'maxLength': 200,
					"source": "/_api_/logs",
					"operation": ['stop', 'restart', 'clear', 'showLineNumber', 'filter']
				}
			},]
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
