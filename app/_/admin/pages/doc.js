(function () {
	const response = {
		data: {
			"type": "page",
			"title": "内容管理",
			"body": [{
				"type": "crud",
				"api": "/_api_/file/query",
				"mode": "list",
				"placeholder": "当前组内, 还没有配置任何权限.",
				"syncLocation": false,
				"title": null,
				"listItem": {
					"title": "$filename",
					"subTitle": "文件路径：${filepath | raw}",
					"actions": [
						{
							"icon": "fa fa-edit",
							"actionType": "drawer",  // 打开抽屉进行文件编辑
							"drawer": {
								"title": "【$filename】",
								"resizable": true,
								"size": "lg",
								"width": "90%",
								"body": [
									{
										"type": "form",
										"api": "/_api_/file/query",
										"initApi": "/_api_/file/query?path=$filepath",
										"controls": [
											{
												"type": "hidden",
												"name": "filepath"
											},
											{
												"type": "editor",  // 文件内容编辑器
												"name": "content",
												"label": "编辑内容",
												"size": "xxl",
												"mode": "code",
												"language": "text",
											},
										],
										"actions": [
											{
												"type": "submit",
												"label": "保存",
												"level": "primary"
											}
										]
									}
								]
							}
						}
					]
				}
			}]
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
