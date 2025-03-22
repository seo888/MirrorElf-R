(function () {
	const response = {
		data:
		{
			"title": "图表示例",
			"body": [
				{
					"type": "grid",
					// "width": "450px",
					"columns": [
						{
							"type": "chart",
							"api": "/_api_/info/qps?count=5",
							"interval": 5000,
							"tracker": true,
							
							xs:7,
							"height": "500px",
							"config":
							{
								tooltip: {
									trigger: 'item',
									formatter: '{a} <br/>{b}: {c} ({d}%)'
								},
								legend: {
									data: [
										'Direct',
										'Marketing',
										'Search Engine',
										'Email',
										'Union Ads',
										'Video Ads',
										'Baidu',
										'Google',
										'Bing',
										'Others'
									]
								},
								graphic: [
									{
									  type: 'text',
									  z:100,
									  left: 'center',
									  top: 'center',
									  style: {
										text: '${qps}', // 标题内容
										textAlign: 'center',
										fill: '#FFFFFF', // 文字颜色
										fontSize: 18, // 文字大小
										fontWeight: 'bold' // 文字加粗
									  }
									}
								  ],
								series: [
									{
										name: 'QPS',
										type: 'pie',
										// selectedMode: 'single',
										radius: [0, '30%'],
										label: {
											position: 'inner',
											fontSize: 14
										},
										labelLine: {
											show: true
										},
										data: [
											{ value: "${qps}", name: '访问 / 秒' },
										]
									},
									{
										name: 'QPS',
										type: 'pie',
										radius: ['45%', '60%'],
										labelLine: {
											length: 30
										},
										label: {
											formatter: '{a|{a}}{abg|}\n{hr|}\n  {b|{b}：}{c}  {per|{d}%}  ',
											backgroundColor: '#F6F8FC',
											borderColor: '#8C8D8E',
											borderWidth: 1,
											borderRadius: 4,
											rich: {
												a: {
													color: '#6E7079',
													lineHeight: 22,
													align: 'center'
												},
												hr: {
													borderColor: '#8C8D8E',
													width: '100%',
													borderWidth: 1,
													height: 0
												},
												b: {
													color: '#4C5058',
													fontSize: 14,
													fontWeight: 'bold',
													lineHeight: 33
												},
												per: {
													color: '#fff',
													backgroundColor: '#4C5058',
													padding: [3, 4],
													borderRadius: 4
												}
											}
										},
										data: "${spider_data}"
									}
								]
							}
						},
						{
							"type": "chart",
							align: "right",
							"height": "455px",
							"width": "450px",
							// "style": {
							// 	"flex": "2" // 占据 8 份
							// },
							"config": {
								series: [
									{
										type: 'gauge',
										startAngle: 90,
										endAngle: -270,
										pointer: {
											show: false
										},
										progress: {
											show: true,
											overlap: true,
											roundCap: true,
											clip: false,
											itemStyle: {
												borderWidth: 1,
												borderColor: '#464646'
											}
										},
										// axisLine: {
										// 	lineStyle: {
										// 		width: 40
										// 	}
										// },
										axisLine: {
											lineStyle: {
												width: 50,
												// color: [
												// 	[0.00, '#008000'], // 0% 为深绿色
												// 	[0.05, '#0d8c0d'], // 5% 为稍浅的绿色
												// 	[0.10, '#1a991a'], // 10% 为更浅的绿色
												// 	[0.15, '#26a626'], // 15% 为浅绿色
												// 	[0.20, '#33b333'], // 20% 为更浅的绿色
												// 	[0.25, '#40bf40'], // 25% 为浅绿色
												// 	[0.30, '#4dcc4d'], // 30% 为更浅的绿色
												// 	[0.35, '#59d959'], // 35% 为浅绿色
												// 	[0.40, '#66e566'], // 40% 为更浅的绿色
												// 	[0.45, '#73f273'], // 45% 为浅绿色
												// 	[0.50, '#80ff80'], // 50% 为非常浅的绿色
												// 	[0.55, '#8cff8c'], // 55% 为接近白色的浅绿色
												// 	[0.60, '#99ff99'], // 60% 为极浅的绿色
												// 	[0.65, '#ff9999'], // 65% 为浅红色
												// 	[0.70, '#ff6666'], // 70% 为稍深的浅红色
												// 	[0.75, '#ff3333'], // 75% 为明亮的红色
												// 	[0.80, '#ff0000'], // 80% 为标准红色
												// 	[0.85, '#cc0000'], // 85% 为深红色
												// 	[0.90, '#990000'], // 90% 为更深的红色
												// 	[0.95, '#660000'], // 95% 为暗红色
												// 	[1.00, '#330000']  // 100% 为接近黑色的深红色
												// ]
											}
										},
										splitLine: {
											show: false,
											distance: 0,
											length: 10
										},
										axisTick: {
											show: false
										},
										axisLabel: {
											show: false,
											distance: 50
										},
										data: [
											{
												value: 20,
												name: 'CPU',
												title: {
													offsetCenter: ['0%', '-40%']
												},
												detail: {
													valueAnimation: true,
													offsetCenter: ['0%', '-25%']
												}
											},
											{
												value: 40,
												name: '内存',
												title: {
													offsetCenter: ['0%', '-7.5%']
												},
												detail: {
													valueAnimation: true,
													offsetCenter: ['0%', '7.5%']
												}
											},
											{
												value: 10,
												name: '硬盘',
												title: {
													offsetCenter: ['0%', '25%']
												},
												detail: {
													valueAnimation: true,
													offsetCenter: ['0%', '40%']
												}
											}
										],
										title: {
											fontSize: 14
										},
										detail: {
											width: 50,
											height: 14,
											fontSize: 14,
											color: 'inherit',
											borderColor: 'inherit',
											borderRadius: 20,
											borderWidth: 1,
											formatter: '{value}%'
										}
									}
								]
							},
							// "api": {
							// 	"method": "GET",
							// 	"url": "/_api_/info/spider_count?days=5",
							// 	"autoLoad": true,
							// 	// "adaptor": "function (payload) { return { data: payload.data }; }"
							// }
						},
						// {
						// 	"title": "QPS",
						// 	"type": "chart",
						// 	"height": "450px",
						// 	"api": "/_api_/info/qps?count=5",
						// 	"interval": 1000,
						// 	"xs": 7
						// },
						
					]
				},
				{
					"title": "蜘蛛概况",
					"height": "450px",
					"type": "chart",
					"api": "/_api_/info/spider_count?days=5",
					// "interval": 5000
				},

			]
		}
		,
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
