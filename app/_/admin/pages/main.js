(function () {
	const response = {
		data:
		// {
		// 	type: "page",
		// 	title: "仪表盘",
		// 	body: [{
		// 		"type": "panel",
		// 		"title": "网站实时日志",
		// 		"body": {
		// 			"type": "log",
		// 			"height": 300,
		// 			// "rowHeight": 22,
		// 			'maxLength': 10000,
		// 			"source": "/_api_/logs",
		// 			"operation": ['stop', 'restart', 'clear', 'showLineNumber', 'filter']
		// 		}
		// 	},]
		// }
		// {
		// 	"type": "page",
		// 	"body": [
		// 	  {
		// 		"type": "log",
		// 		"id": "logComponent",
		// 		"height": 300,
		// 		"maxLength": 10000,
		// 		"source":'/_api_/logs',
		// 		"operation": ["stop", "restart", "clear", "showLineNumber", "filter"]
		// 	  },
		// 	],

		// }
		{
			"title": "图表示例",
			"body": [
				{
					"type": "grid",
					"columns": [
						{
							"type": "panel",
							"title": "本地配置示例 支持交互",
							"name": "chart-local",
							"body": [
								{
									"type": "chart",
									"config": {
										"title": {
											"text": "极坐标双数值轴"
										},
										"legend": {
											"data": [
												"line"
											]
										},
										"polar": {
											"center": [
												"50%",
												"54%"
											]
										},
										"tooltip": {
											"trigger": "axis",
											"axisPointer": {
												"type": "cross"
											}
										},
										"angleAxis": {
											"type": "value",
											"startAngle": 0
										},
										"radiusAxis": {
											"min": 0
										},
										"series": [
											{
												"coordinateSystem": "polar",
												"name": "line",
												"type": "line",
												"showSymbol": false,
												"data": [
													[
														0,
														0
													],
													[
														0.03487823687206265,
														1
													],
													[
														0.06958655048003272,
														2
													],
													[
														0.10395584540887964,
														3
													],
													[
														0.13781867790849958,
														4
													],
													[
														0.17101007166283433,
														5
													],
													[
														0.2033683215379001,
														6
													],
													[
														0.2347357813929454,
														7
													],
													[
														0.26495963211660245,
														8
													],
													[
														0.2938926261462365,
														9
													],
													[
														0.3213938048432697,
														10
													]
												]
											}
										],
										"animationDuration": 2000
									},
									"clickAction": {
										"actionType": "dialog",
										"dialog": {
											"title": "详情",
											"body": [
												{
													"type": "tpl",
													"tpl": "<span>当前选中值 ${value|json}<span>"
												},
												{
													"type": "chart",
													"api": "/amis/api/mock2/chart/chart1"
												}
											]
										}
									}
								}
							]
						},
						{
							"type": "panel",
							"title": "远程图表示例(返回值带function)",
							"name": "chart-remote",
							"body": [
								{
									"type": "chart",
									"api": "/amis/api/mock2/chart/chart1"
								}
							]
						}
					]
				},
				{
					"type": "service",
					// "api": {
					//   "method": "get",
					//   "url": "/_api_/logs",
					//   "autoRefresh": false,
					//   "concatDataFields": "log"
					// },
					// "silentPolling": true,
					// "interval": 1000,
					// "stopAutoRefreshWhen": "${finished}",
					"body": [
						{
							"type": "log",
							"height": 300,
							"maxLength": 10000,
							"source": {
							  "method": "get",
							  "url": "/_api_/logs",
							  "adaptor": "(payload) => { return { ...payload, status: 0 }; }"
							},
							"interval": 5000, // 每隔 5 秒重新加载日志
							"operation": ["stop", "restart", "clear", "showLineNumber", "filter"],
							"onEvent": {
							  "restart": {
								"actions": [
								  {
									"actionType": "reload", // 重新加载日志
									"componentId": "logComponent" // 替换为你的 log 组件 ID
								  }
								]
							  }
							}
						  }
					]
				}

			]
		}
		,
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
