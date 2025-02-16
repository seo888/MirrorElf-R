(function () {
	const response = {
		data: {
			"type": "page",
			"title": "设置",
			"body": [
				{
					"type": "form",
					"name": "set_form",
					"mode": "horizontal",
					"labelWidth": 220,
					"api": {
						"method": "put",
						"url": "/_api_/config",
						"requestAdaptor": function (api) {
							if (api.data && typeof api.data === 'object') {
								Object.keys(api.data).forEach(function (key) {
									if (typeof api.data[key] === 'string') {
										api.data[key] = api.data[key].replace(/<script/g, '<3cript');
									}
								});
							}
							return api;
						}
					},
					"reload": "set_form",
					"initApi": "/_api_/config",
					"actions": [
						{
							"type": "tpl",
							"tpl": "${act_info} <a href='https://t.me/MirrorElf' target='_blank'>续费</a>"
						},
						{
							"label": "设备机器码：${machine_id}",
							"type": "button",
							"level": "link",
							"actionType": "copy",
							"content": "${machine_id}",
							"tooltip": "点击复制",
							"tooltipPlacement": "top"
						},
						{
							"type": "submit",
							"level": "info",
							"icon": "fa fa-save",
							"label": "保存"  // 修改提交按钮文本
						},],
					"title": "",
					"body": [
						{
							"type": "anchor-nav",
							"direction": "horizontal",
							"style": {
								"height": "76vh",
							},
							"links": [
								{
									"title": "网站信息",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "网站信息",
											"body": [
												{
													"name": "程序名称",
													"type": "input-text",
													"label": "程序名称"
												},
												{
													"name": "授权码",
													"type": "input-text",
													"label": "授权码",
													"required": true,
													"desc": "新服务器赠送1天时间的授权码，发送“右下角-设备机器码”至 https://t.me/MirrorElf 领取免费授权码"
												},
												{
													"name": "登录账号",
													"type": "input-text",
													"label": "登录账号",
													"required": true,
												},
												{
													"name": "登录密码",
													"type": "input-password",
													"label": "登录密码",
													"required": true,
													"hint": "默认账号密码admin，请及时修改"
												},
												{
													"name": "雷池Token",
													"type": "input-text",
													"label": "雷池Token",
													"desc": "与防火墙通信，展示网站数据、自动https证书，请务必正确填写"
												}
											]
										}
									]
								},
								{
									"title": "网站设置",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "网站设置",
											"body": [
												{
													"name": "语言",
													"type": "radios",
													"label": "语言",
													"value": 3,
													"options": [
														{
															"label": "中文",
															"value": 'zh'
														},
														{
															"label": "英文",
															"value": 'en'
														},
														{
															"label": "葡萄牙文",
															"value": 'pt',
														}
													]
												},
												{
													"name": "自动建站",
													"type": "switch",
													"label": "自动建站"
												},
												{
													"name": "自动https证书",
													"type": "switch",
													"label": "自动https证书"
												},
												{
													"name": "泛站自动建站",
													"type": "switch",
													"label": "泛站自动建站"
												},
												{
													"name": "泛站爬取目标",
													"type": "switch",
													"label": "泛站爬取目标"
												},
												{
													"name": "缓存静态文件",
													"type": "input-text",
													"label": "缓存静态文件"
												},
												{
													"name": "首页更新时间",
													"type": "input-number",
													"label": "首页更新时间",
													"required": true,
													"desc": "单位：天 填写0则永不更新首页"
												},
												{
													"name": "链接映射",
													"type": "switch",
													"label": "链接映射"
												}
											]
										}
									]
								},
								{
									"title": "目录功能",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "目录功能",
											"body": [
												{
													"name": "模板模式",
													"type": "radios",
													"label": "模板模式",
													"value": 4,
													"options": [
														{
															"label": "关闭: 不调用模板",
															"value": '关闭'
														},
														{
															"label": "404: 目标站404时返回模板",
															"value": '404'
														},
														{
															"label": "泛目录: 所有内页调用模板，缓存页面",
															"value": '泛目录',
														},
														{
															"label": "蜘蛛池: 所有内页调用模板，不缓存页面",
															"value": '蜘蛛池'
														}
													]
												},
												{
													"name": "TDK格式",
													"type": "input-text",
													"label": "TDK格式"
												},
												{
													"name": "地图链接格式",
													"type": "input-text",
													"label": "地图链接格式",
												},
												{
													"name": "地图链接数量",
													"type": "input-number",
													"label": "地图链接数量",
													"required": true
												}
											]
										}
									]
								},
								{
									"title": "SEO功能",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "SEO功能",
											"body": [
												{
													"name": "外链策略",
													"type": "radios",
													"label": "外链策略",
													"value": 4,
													"options": [
														{
															"label": "不处理",
															"value": '0'
														},
														{
															"label": "转为本站内部链接",
															"value": '1'
														},
														{
															"label": "转为全站随机链接（主站+泛站）",
															"value": '2',
														},
														{
															"label": "转为链轮链接（服务器上所有站点）",
															"value": '3',
														}
													]
												},
												{
													"name": "图片翻转",
													"type": "switch",
													"label": "图片翻转"
												},
												{
													"name": "随机meta&link",
													"type": "switch",
													"label": "随机meta&link"
												},
												{
													"name": "随机class属性",
													"type": "switch",
													"label": "随机class属性"
												},
												{
													"name": "js加密混淆",
													"type": "switch",
													"label": "js加密混淆"
												},
												{
													"name": "css加密混淆",
													"type": "switch",
													"label": "css加密混淆"
												}
											]
										}
									]
								},
								{
									"title": "访问策略",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "访问策略",
											"body": [
												{
													"name": "非绑定域名来路",
													"type": "switch",
													"label": "非绑定域名来路"
												},
												{
													"name": "IP与非域名来路",
													"type": "switch",
													"label": "IP与非域名来路"
												},
												{
													"name": "泛站来路",
													"type": "switch",
													"label": "泛站来路"
												},
												{
													"name": "UA黑名单",
													"type": "textarea",
													"label": "UA黑名单"
												},
												{
													"name": "IP黑名单",
													"type": "textarea",
													"label": "IP黑名单"
												}
											]
										}
									]
								},
								{
									"title": "广告策略",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "广告策略",
											"body": [
												{
													"name": "搜索来路跳广告",
													"type": "switch",
													"label": "搜索来路跳广告"
												},
												{
													"name": "普通用户跳广告",
													"type": "switch",
													"label": "普通用户跳广告"
												},
												{
													"name": "广告URL",
													"type": "input-text",
													"label": "广告URL"
												}
											]
										}
									]
								},
								{
									"title": "JS代码插入",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "JS代码插入",
											"body": [
												{
													"name": "过滤IP",
													"type": "input-text",
													"label": "过滤IP"
												},
												{
													"name": "head头部",
													"type": "textarea",
													"label": "head头部"
												},
												{
													"name": "head尾部",
													"type": "textarea",
													"label": "head尾部"
												},
												{
													"name": "body头部",
													"type": "textarea",
													"label": "body头部"
												},
												{
													"name": "body尾部",
													"type": "textarea",
													"label": "body尾部"
												}
											]
										}
									]
								},
								{
									"title": "蜘蛛策略",
									"body": [
										{
											"type": "fieldSet",
											"collapsable": true,
											"title": "蜘蛛策略",
											"body": [
												{
													"name": "谷歌蜘蛛",
													"type": "switch",
													"label": "谷歌蜘蛛"
												},
												{
													"name": "百度蜘蛛",
													"type": "switch",
													"label": "百度蜘蛛"
												},
												{
													"name": "必应蜘蛛",
													"type": "switch",
													"label": "必应蜘蛛"
												}
											]
										}
									]
								}
							]
						}
					]
				}
			]
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
