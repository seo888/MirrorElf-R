(function () {
	const response = {
		data: {
			"type": "page",
			// "title": "网站管理",
			// "toolbar": [

			// ],

			"body": {
				"type": "crud",
				"itemBadge": {
					"text": "${website_info.subdomain == 'www' ? '主站' : '泛站'}",
					// "variations": {
					// 	"true": "primary",
					// 	"false": "danger"
					// },
					"mode": "ribbon",
					// "offset": [
					// 			-20,
					// 			0
					// 		],
					"position": "top-left",
					"level": "${website_info.subdomain == 'www' ? 'info' : 'danger'}",
					// "visibleOn": "this.is_www"
				},
				"onEvent": {
					"selectedChange": {
						"actions": [
							{
								"actionType": "toast",
								"args": {
									"msg": "已选择${event.data.selectedItems.length}条记录"
								}
							}
						]
					}
				},
				"id": "crud-table",
				"syncLocation": false,
				// "quickSaveApi": "/_api_/website/update?id=${id}",  // 更新 API 地址
				// "deferApi": "/_api_/website/query?parentId=${id}",
				// "draggable": true,
				"api": "/_api_/website/query",
				// "checkOnItemClick": true,
				"perPageAvailable": [
					10,
					20,
					100,
					500,
				],
				"perPage": 10,
				"keepItemSelectionOnPageChange": true,
				"autoFillHeight": true,
				"labelTpl": "【${id}】${domain}",
				"autoGenerateFilter": {
					"columnsNum": 6,
					"showBtnToolbar": true
				},
				"bulkActions": [
					{
						"label": "批量删除",
						"level": "danger",
						"actionType": "ajax",
						"api": "delete:/_api_/website/delete?files=${ids|raw}",
						"confirmText": "确认批量删除网站【${ids|raw}】（注意：操作不可逆，请谨慎操作）"
					},
					{
						"label": "批量复制",
						"type": "button",
						"onClick": "console.log(props.data.selectedItems); const rows = props.data.selectedItems; if (rows && rows.length) { const textToCopy = rows.map(row => row.domain ? row.domain : '').join('\\n'); const textArea = document.createElement('textarea'); textArea.value = textToCopy; document.body.appendChild(textArea); textArea.select(); document.execCommand('copy'); document.body.removeChild(textArea); props.env.notify('success', '已复制以下域名到剪贴板：\\n' + textToCopy);}"
					}

				],
				// "quickSaveApi": "/amis/api/sample/bulkUpdate",
				// "quickSaveItemApi": "/amis/api/sample/$id",
				"filterTogglable": true,
				"headerToolbar": [
					"bulkActions",
					"export-excel",
					{
						"type": "button",
						"actionType": "dialog",
						"label": "建站",
						"icon": "fa fa-plus pull-left",
						"primary": true,
						"dialog": {
							"resizable": true,
							"size": "lg",
							"title": "新建网站",
							"body": {
								"type": "form",
								"size": "lg",
								"name": "sample-edit-form",
								"api": "post:/_api_/website/insert",
								"reload": "crud-table", // 在提交后重新加载特定的组件
								"body": [
									{
										"type": "divider",
										"title": "【网站设置】",
										"titlePosition": "center"
									},
									{
										"type": "group",
										"body": [
											{
												"type": "input-text",
												"name": "domain",
												"label": "域名",
												"required": true,
												"validations": {
													"matchRegexp": "^(?!https?://)([\\w-]+\\.)+[\\w-]{2,}$"  // 正则表达式，确保不包含 http 头
												},
												"validationErrors": {
													"matchRegexp": "请输入有效的纯域名，不带http头"
												},
												"placeholder": "请输入纯域名，不带http头 例如: www.abc.com"
											},
											{
												"type": "group",
												"body": [
													{
														"type": "select",
														"name": "lang",
														"label": "语言",
														"options": [
															{
																"label": "中文",
																"value": "zh"
															},
															{
																"label": "英文",
																"value": "en"
															}
														],
														"value": "zh",  // 设置默认值为 zh
														"placeholder": "请选择语言"
													},
													{
														name: "homepage_update_time",
														type: "input-number",
														label: "首页更新时间",
														width: "80px",
														value: 0,
														required: true,
														desc: "单位：天 填0关闭"
													}]
											}]
									},
									{
										"type": "input-text",
										"name": "title",
										"label": "网站标题",
										"placeholder": "请输入网站标题",
										"required": true
									},
									{
										"type": "input-text",
										"name": "keywords",
										"label": "关键词",
										"placeholder": "请输入网站关键词（以,号隔开）",
										"required": true
									},
									{
										"type": "textarea",
										"name": "description",
										"label": "描述",
										"placeholder": "请输入描述内容",
										"minRows": 3,  // 可选，指定最少显示的行数
										"maxRows": 6,   // 可选，指定最多显示的行数
										"required": true
									},
									{
										"type": "group",
										"body": [
											{
												"type": "select",
												"name": "replace_mode",
												"label": "替换模式",
												"options": [
													{
														"label": "0. 仅目标站替换",
														"value": 0
													},
													{
														"label": "1. 先 目标站替换 后 本站替换",
														"value": 1
													},
													{
														"label": "2. 仅本站替换",
														"value": 2
													},
													{
														"label": "3. 先 本站替换 后 目标站替换",
														"value": 3
													},
												],
												"value": 0,  // 设置默认值为 zh
												// "placeholder": "是否开启链接映射"
											},
											{
												"type": "select",
												"name": "link_mapping",
												"label": "链接映射",
												// "required": true,
												"options": [
													{
														"label": "开启",
														"value": true
													},
													{
														"label": "关闭",
														"value": false
													}
												],
												"value": false,  // 设置默认值为 zh
												"placeholder": "是否开启链接映射"
											}]
									},
									{
										"type": "divider",
										"title": "【替换规则】",
										"titlePosition": "center"
									},
									{
										"type": "input-text",
										"name": "target",
										"label": "目标站",
										"required": true,
										"validations": {
											"matchRegexp": ".*\\|.*"
										},
										"validationErrors": {
											"matchRegexp": "请使用间隔符“|” 指定目标站语言 如: en|www.english.com  或  zh|www.chinese.com"
										},
										"placeholder": "目标站格式: en|www.english.com"
									},
									// 插入新的 service，用于加载 target_replace 数据
									{
										"type": "service",
										"api": "/_api_/replace/query?domain=$target",  // 动态加载 target_replace 数据的 API
										"body": [
											{
												"type": "editor",
												"language": "yaml",
												"name": "target_replace",
												"label": "目标站替换",
												"value": "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'"
											}
										]
									},
									{
										"type": "alert",
										"body": "注意：替换词格式按照“先长后短”方式，如“hello world -> {关键词}”在上，“hello -> 你好”在下",
									},

									{
										"type": "input-array",
										"name": "replace_rules_all",
										"label": "全局替换",
										"items": {
											"type": "input-text",
											"name": "-",
											"label": "-",
											"unique": true
										},
										"addButtonText": "规则",
										"scaffold": "待替换字符串 -> {keyword}",
										"minItems": 0,
									},
									{
										"type": "input-array",
										"name": "replace_rules_index",
										"label": "首页替换",
										"items": {
											"type": "input-text",
											"name": "-",
											"label": "-",
											"unique": true
										},
										"addButtonText": "规则",
										"minItems": 0,
									},
									{
										"type": "input-array",
										"name": "replace_rules_page",
										"label": "内页替换",
										"items": {
											"type": "input-text",
											"name": "-",
											"label": "-",
											"unique": true
										},
										"addButtonText": "规则",
										"minItems": 0,
									},
									{
										"type": "divider",
										"title": "【泛目录配置】",
										"titlePosition": "center"
									},
									{
										name: "mulu_tem_max",
										type: "input-number",
										label: "生成模板数量",
										required: true,
										"value": 0,
										desc: "填写0则不会自动生成模板"
									},
									{
										"type": "select",
										"name": "mulu_static",
										"label": "泛目录模式",
										"options": [
											{
												"label": "静态",
												"value": true
											},
											{
												"label": "动态（蜘蛛池）",
												"value": false
											}
										],
										"value": true,
									},
									{
										type: "checkboxes",
										name: "mulu_mode",
										label: "泛目录路由",
										checkAll: true,
										optionType: "button",
										options: [
											{ label: "404页面", value: "404" },
											{ label: "非首页（所有页面）", value: "all_page" },
											{ label: "自定义路径", value: "custom_header" },
										]
									},
									{
										"type": "input-array",
										"name": "mulu_custom_header",
										"label": "自定义路径",
										"items": {
											"type": "input-text",
											"name": "/",
											"label": "/",
											"unique": true,
										},
										"addButtonText": "泛目录路径",
										"minItems": 0,
									},
									{
										"type": "input-array",
										"name": "mulu_keywords_file",
										"label": "关键词库",
										"items": {
											"type": "input-text",
											"name": "词库路径",
											"label": "词库路径",
											"unique": true,
										},
										"addButtonText": "关键词库",
										"minItems": 0,
									},
								]
							}
						}
					},
					{
						"type": "button",
						"label": "批量建站",
						"icon": "fa fa-plus pull-left",
						"primary": true,
						"actionType": "drawer",
						"drawer": {
							"resizable": true,
							"size": "lg",
							"width": "90%",
							"title": "批量建站",
							"body": {
								"type": "form",
								"name": "sample-edit-form",
								"api": "/_api_/website/create",
								"reload": "crud-table",
								"body": [
									{
										"type": "divider",
										"title": "【建站策略】",
										"titlePosition": "center"
									},
									{
										"type": "group",
										"body": [
											{
												"type": "select",
												"name": "over_write",
												"label": "建站模式",
												"options": [
													{
														"label": "覆盖已有网站",
														"value": true
													},
													{
														"label": "跳过已有网站",
														"value": false
													}
												],
												"value": false,
												"placeholder": "是否覆盖"
											},
											{
												"type": "select",
												"name": "target_replace_over_write",
												"label": "目标站替换词",
												"options": [
													{
														"label": "存在则强制覆盖",
														"value": true
													},
													{
														"label": "存在则跳过",
														"value": false
													}
												],
												"value": false,
												"placeholder": "是否覆盖"
											},]
									},
									{
										"type": "divider",
										"title": "【网站设置】",
										"titlePosition": "center"
									},
									{
										"type": "group",
										"body": [
											{
												"type": "select",
												"name": "replace_mode",
												"label": "替换模式",
												"options": [
													{
														"label": "0. 仅目标站替换",
														"value": 0
													},
													{
														"label": "1. 先 目标站替换 后 本站替换",
														"value": 1
													},
													{
														"label": "2. 仅本站替换",
														"value": 2
													},
													{
														"label": "3. 先 本站替换 后 目标站替换",
														"value": 3
													},
												],
												"value": 0,
											},
											{
												"type": "select",
												"name": "link_mapping",
												"label": "链接映射",
												"options": [
													{
														"label": "开启",
														"value": true
													},
													{
														"label": "关闭",
														"value": false
													}
												],
												"value": false,
												"placeholder": "是否开启链接映射"
											},
											{
												type: "input-number",
												name: "homepage_update_time",
												label: "首页更新时间",
												required: true,
												desc: "单位：天 填0关闭",
												"value": 0,  // 设置默认值
											},
											{
												"type": "select",
												"name": "lang",
												"label": "语言",
												// "required": true,
												"options": [
													{
														"label": "中文",
														"value": "zh"
													},
													{
														"label": "英文",
														"value": "en"
													}
												],
												"value": "zh",  // 设置默认值为 zh
												"placeholder": "请选择语言"
											},
										]
									},


									// {
									// 	"type": "alert",
									// 	"body": "格式：<域名>__<目标站>__<链接映射(true/false)>__<标题>__<关键词>__<描述>__<替换模式(0/1/2/3)>__<目标站替换词(可留空)>__<本站替换词(可留空)>"
									// },
									{
										"type": "alert",
										"body": "例子：www.domain.com___en|www.target.com___网站标题___网站关键词___网站描述___关于我们----------{keyword}##########公司名称----------【关键词】___关于我们 -> {keyword} ; 公司名称 -> 【关键词】"
									},

									{
										"type": "button",
										"className": "pull-right",
										"label": "清空",
										"onEvent": {
											"click": {
												"actions": [
													{
														"actionType": "clear",
														"componentId": "content"
													}
												]
											}
										}
									},
									{
										"type": "button",
										"icon": "fa fa-plus",
										"level": "link",
										"label": "加载预建站文档",
										"actionType": "ajax",
										"api": "get:/_api_/file/query?path=doc/website.txt",
										"messages": {
											"success": "加载成功",
											"failed": "加载失败"
										},
									},
									{
										"type": "editor",
										"language": "yaml",
										"name": "content",
										"id": "content",
										"label": "建站信息",
										"placeholder": "<域名>___<目标站>___<标题>___<关键词>___<描述>___<目标站替换词(可留空)>___<本站替换词(可留空)>",
										"value": "",
									},
									{
										"type": "alert",
										"level": "info",
										"showIcon": true,
										"body": "标准格式： 间隔符为\" -> \"，多组分隔符为\" ; \"，如：关于我们 -> {keyword} ; 公司名称 -> 【关键词】"
									},
									{
										"type": "alert",
										"level": "info",
										"showIcon": true,
										"body": "兼容格式： 间隔符为\"----------\"，多组分隔符为\"##########\"，如：关于我们----------{keyword}##########公司名称----------【关键词】"
									}
									,
									{
										"type": "divider",
										"title": "【泛目录配置】",
										"titlePosition": "center"
									},
									{
										type: "checkboxes",
										name: "mulu_mode",
										label: "泛目录路由",
										checkAll: true,
										optionType: "button",
										options: [
											{ label: "404页面", value: "404" },
											{ label: "非首页（所有页面）", value: "all_page" },
											{ label: "自定义路径", value: "custom_header" },
										]
									},
									{
										"type": "group",
										"body": [
											{
												name: "mulu_tem_max",
												type: "input-number",
												label: "生成模板数量",
												required: true,
												value: 0,
												desc: "填写0则不会自动生成模板"
											},
											{
												"type": "select",
												"name": "mulu_static",
												"label": "泛目录模式",
												"options": [
													{
														"label": "静态",
														"value": true
													},
													{
														"label": "动态（蜘蛛池）",
														"value": false
													}
												],
												"value": true,
											}]
									},
									{
										"type": "group",
										"body": [
											{
												"type": "input-array",
												"name": "mulu_custom_header",
												"label": "自定义路径",
												"items": {
													"type": "input-text",
													"name": "/",
													"label": "/",
													"unique": true,
												},
												"addButtonText": "泛目录路径",
												"minItems": 0,
											},
											{
												"type": "input-array",
												"name": "mulu_keywords_file",
												"label": "关键词库",
												"items": {
													"type": "input-text",
													"name": "词库路径",
													"label": "词库路径",
													"unique": true,
												},
												"addButtonText": "关键词库",
												"minItems": 0,
											},]
									},
								]
							}
						}
					},
					{
						"type": "tpl",
						"tpl": "主站: ${www_count} | 泛站: ${web_count} | 共: ${www_count+web_count}",
						"className": "v-middle"
					},
					"reload",
					{
						"type": "columns-toggler",
						"align": "right"
					},
					{
						"type": "pagination",
						"align": "right"
					},
					{
						"type": "tpl",
						"tpl": "当前：${items_count} 项 | 共：${count} 项",
						"align": "right"
					},

				],
				"footerToolbar": [
					"statistics",
					{
						"type": "pagination",
						"layout": "perPage,pager,go"
					}
				],
				"columns": [
					{
						"type": "tpl",
						"name": "id",
						"label": "ID",
						"searchable": {
							"type": "textarea",
							"name": "search_term",
							"label": "🔍搜索",
							"clearable": true,
							"maxLength": 10000,
							"showCounter": true,
						},
						// "width": 80,
						// "badge": {
						// 	"mode": "text",
						// 	// "animation": true,
						// 	"size": 12,
						// 	"offset": [
						// 		15,
						// 		0
						// 	],
						// 	"visibleOn": "this.children && this.children.length > 0",
						// 	"overflowCount": 999999,
						// 	"text": "${children.length}",
						// },
						"fixed": "left",
						"sortable": true,  // 启用排序功能
					},
					// {
					// 	"name": "id",
					// 	"label": "文件路径",
					// 	"searchable": {
					// 		"type": "textarea",
					// 		"name": "search_term",
					// 		"label": "🔍搜索",
					// 		"clearable": true,
					// 		"maxLength": 10000,
					// 		"showCounter": true,
					// 	},
					// 	// "fixed": "left",
					// 	// "sortable": true,  // 启用排序功能
					// 	"visible": false
					// },
					{
						"type": "static-mapping",
						"name": "website_info.subdomain",
						"label": "站点类型",
						"visible": false,
						// "sortable": true,
						"searchable": {
							"type": "select",
							"name": "is_www",
							"label": "站点类型",
							"options": [
								{
									"label": "主站+泛站",
									"value": 0
								},
								{
									"label": "主站",
									"value": 1
								},
								{
									"label": "泛站",
									"value": 2
								}
							],
							"value": 0,  // 默认值设置为 "主站+泛站"
							"placeholder": "选择站点类型"
						}
					},
					{
						"type": "tpl",
						"tpl": "<a href='http://${website_info.domain}' target='_blank' class='link-style'>${website_info.domain}</a>",
						"name": "website_info.domain",
						"label": "域名",
						"fixed": "left",
						"copyable": true,
						"searchable": {
							"name": "domain",
							"clearable": true,
							"maxLength": 1000,
						},
						// "sortable": true
					},
					{
						"name": "website_info.to_lang",
						"label": "语言",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"name": "website_info.root_domain",
						"label": "根域名",
						"copyable": true,
						"popOver": {
							"trigger": "hover",
							"body": {
								"type": "tpl",
								"tpl": "${website_info.root_domain} 查收录：<a href='https://www.google.com/search?q=site%3A${website_info.root_domain}' target='_blank' class='link-style' title='site:${website_info.root_domain}'>谷歌</a> | <a href='https://www.bing.com/search?q=site%3A${website_info.root_domain}' target='_blank' class='link-style' title='site:${website_info.root_domain}'>必应</a> | <a href='https://www.baidu.com/s?wd=site%3A${website_info.root_domain}' target='_blank' class='link-style' title='site:${website_info.root_domain}'>百度</a> | <a href='https://www.sogou.com/web?query=site%3A${website_info.root_domain}' target='_blank' class='link-style' title='site:${website_info.root_domain}'>搜狗</a>"
							}
						},
						"sortable": {
							"orderBy": "root_domain"
						},
						"searchable": {
							"name": "root_domain",
							"clearable": true,
							"maxLength": 1000,
						},
					},
					{
						"type": "tpl",
						"tpl": "<a href='javascript:void(0);' class='link-icon'>${website_info.target}</a>",
						"name": "website_info.target",
						"label": "目标站",
						// "sortable": true,
						"copyable": true,
						"searchable": {
							"name": "target",
							"clearable": true,
							"maxLength": 1000,
						},
						"onEvent": {
							"click": {
								"actions": [
									{
										"actionType": "custom",
										"script": "const parts = event.data.website_info.target.split('|'); if(parts.length > 1) { const linkTarget = parts[1]; window.open('http://' + linkTarget, '_blank'); }"
									}
								]
							}
						}
					},
					{
						"name": "website_info.title",
						"label": "网站标题",
						"copyable": true,
						"popOver": {
							"trigger": "hover",
							"body": {
								"type": "tpl",
								"tpl": "${website_info.domain} 查标题排名：<a href='https://www.google.com/search?q=${website_info.title}' target='_blank' class='link-style' title='${website_info.title}'>谷歌</a> | <a href='https://www.bing.com/search?q=${website_info.title}' target='_blank' class='link-style' title='${website_info.title}'>必应</a> | <a href='https://www.baidu.com/s?wd=${website_info.title}' target='_blank' class='link-style' title='${website_info.title}'>百度</a> | <a href='https://www.sogou.com/web?query=${website_info.title}' target='_blank' class='link-style' title='${website_info.title}'>搜狗</a>"
							}
						}
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"name": "website_info.keywords",
						"label": "关键词",
						"copyable": true,
						"popOver": {
							"trigger": "hover",
							"body": {
								"type": "tpl",
								"tpl": "${website_info.domain} 查关键词排名：<a href='https://www.google.com/search?q=${website_info.keywords | split:',' | first}' target='_blank' class='link-style' title='${website_info.keywords | split:',' | first}'>谷歌</a> | <a href='https://www.bing.com/search?q=${website_info.keywords | split:',' | first}' target='_blank' class='link-style' title='${website_info.keywords | split:',' | first}'>必应</a> | <a href='https://www.baidu.com/s?wd=${website_info.keywords | split:',' | first}' target='_blank' class='link-style' title='${website_info.keywords | split:',' | first}'>百度</a> | <a href='https://www.sogou.com/web?query=${website_info.keywords | split:',' | first}' target='_blank' class='link-style' title='${website_info.keywords | split:',' | first}'>搜狗</a>"
							  }
						}
					},
					{
						"name": "website_info.description",
						"label": "描述",
						"copyable": true,
					},
					// {
					// 	"name": "replace_string",
					// 	"label": "本站替换词",
					// 	"hidden": true  // 隐藏该字段
					// },
					{
						"type": "datetime",  // 显示为日期时间类型
						"name": "updated_at",
						"label": "更新于",
						"width": 150,
						"sortable": true,  // 启用排序功能
					},
					{
						"type": "operation",
						"fixed": "right",
						"label": "操作",
						// "width": 160,
						"width": 110,
						"buttons": [
							// {
							// 	"type": "button",
							// 	"icon": "fa fa-refresh text-danger",
							// 	"actionType": "ajax",
							// 	"tooltipPlacement": "top",
							// 	"tooltip": "换目标站",
							// 	"confirmText": "确认随机更换【${id}】${domain} 目标站: ${target}",
							// 	"api": "get:/_api_/website/random_target?id=$id"
							// },
							// {
							// 	"type": "button",
							// 	"icon": "fa fa-times text-danger",
							// 	"actionType": "ajax",
							// 	"tooltipPlacement": "top",
							// 	"tooltip": "删除目标站",
							// 	"confirmText": "确认删除 目标站库中的: ${target}",
							// 	"api": "delete:/_api_/file/config/target.txt?line=$target",
							// 	"reload": "none"
							// },
							{
								"type": "button",
								"icon": "fa fa-pencil",
								"tooltipPlacement": "top",
								"tooltip": "编辑",
								"actionType": "drawer",
								"drawer": {
									"resizable": true,
									"size": "lg",
									"width": "50%",
									"title": "编辑【$website_info.domain】",
									"body": {
										"type": "form",
										"name": "sample-edit-form",
										"api": "put:/_api_/website/update?id=$id",
										"reload": "crud-table", // 在提交后重新加载特定的组件
										"body": [
											{
												"type": "static",
												"name": "id",
												"label": "ID",
												"visible": false
											},
											{
												"type": "divider",
												"title": "【网站设置】",
												"titlePosition": "center"
											},

											{
												"type": "group",
												"body": [
													{
														"type": "static",
														"name": "website_info.domain",
														"label": "域名",
													},
													{
														"type": "select",
														"name": "website_info.to_lang",
														"label": "语言",
														"options": [
															{
																"label": "中文",
																"value": "zh"
															},
															{
																"label": "英文",
																"value": "en"
															}
														],
														"placeholder": "请选择语言"
													},
												]
											},
											{
												"type": "group",
												"body": [
													{
														"type": "static",
														"name": "website_info.root_domain",
														"label": "根域名",
													},
													{
														name: "homepage_update_time",
														type: "input-number",
														label: "首页更新时间",
														required: true,
														desc: "单位：天 填0关闭"
													},
												]
											},
											{
												"type": "input-text",
												"name": "website_info.title",
												"label": "网站标题",
												"required": true
											},
											{
												"type": "input-text",
												"name": "website_info.keywords",
												"label": "关键词"
											},
											{
												"type": "textarea",
												"name": "website_info.description",
												"label": "描述"
											},
											{
												"type": "group",
												"body": [
													{
														"type": "select",
														"name": "replace_rules.replace_mode",
														"label": "替换模式",
														"options": [
															{
																"label": "仅 目标站替换",
																"value": 0
															},
															{
																"label": "先 目标站替换 后 本站替换",
																"value": 1
															},
															{
																"label": "仅 本站替换",
																"value": 2
															},
															{
																"label": "先 本站替换 后 目标站替换",
																"value": 3
															},
														],
														// "value": "false",  // 设置默认值为 zh
														// "placeholder": "是否开启链接映射"
													},
													{
														"type": "select",
														"name": "website_info.link_mapping",
														"label": "链接映射",
														"options": [
															{
																"label": "开启",
																"value": true
															},
															{
																"label": "关闭",
																"value": false
															}
														],
														// "value": "false",  // 设置默认值为 zh
														// "placeholder": "是否开启链接映射"
													},
												]
											},
											{
												"type": "divider",
												"title": "【替换规则】",
												"titlePosition": "center"
											},
											{
												"type": "input-text",
												"name": "website_info.target",
												"label": "目标站",
												"required": true,
												"placeholder": "目标站格式: en|www.english.com",
												"validations": {
													"matchRegexp": ".*\\|.*"  // 正则表达式：要求输入中必须包含 "|"
												},
												"validationErrors": {
													"matchRegexp": "请使用间隔符“|” 指定目标站语言 如: en|www.english.com  或  zh|www.chinese.com"  // 自定义错误提示信息
												}
											},

											// 插入新的 service，用于加载 target_replace 数据
											{
												"type": "service",
												"api": "/_api_/replace/query?domain=$website_info.target",  // 动态加载 target_replace 数据的 API
												"body": [
													{
														"type": "editor",
														"language": "yaml",
														"name": "target_replace",
														"label": "目标站替换",
														"value": "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'"
													}
												]
											},
											{
												"type": "alert",
												"level": "info",
												"showIcon": true,
												"body": "注意：替换词格式按照“先长后短”方式，如“hello world -> {关键词}”在上，“hello -> 你好”在下",
											},

											// {
											// 	"type": "editor",
											// 	"language": "yaml",
											// 	"name": "replace_string",
											// 	"label": "本站替换词"
											// },
											{
												"type": "input-array",
												"name": "replace_rules.replace_rules_all",
												"label": "全局替换",
												"items": {
													"type": "input-text",
													"name": "-",
													"label": "-",
													"unique": true
												},
												"addButtonText": "规则",
												"scaffold": "待替换字符串 -> {keyword}",
												"minItems": 0,
											},
											{
												"type": "input-array",
												"name": "replace_rules.replace_rules_index",
												"label": "首页替换",
												"items": {
													"type": "input-text",
													"name": "-",
													"label": "-",
													"unique": true
												},
												"addButtonText": "规则",
												"minItems": 0,
											},
											{
												"type": "input-array",
												"name": "replace_rules.replace_rules_page",
												"label": "内页替换",
												"items": {
													"type": "input-text",
													"name": "-",
													"label": "-",
													"unique": true
												},
												"addButtonText": "规则",
												"minItems": 0,
											},
											{
												"type": "divider",
												"title": "【泛目录配置】",
												"titlePosition": "center"
											},
											{
												name: "mulu_config.mulu_tem_max",
												type: "input-number",
												label: "生成模板数量",
												required: true,
												desc: "填写0则不会自动生成模板"
											},
											{
												"type": "select",
												"name": "mulu_config.mulu_static",
												"label": "泛目录模式",
												"options": [
													{
														"label": "静态",
														"value": true
													},
													{
														"label": "动态（蜘蛛池）",
														"value": false
													}
												],
											},
											{
												type: "checkboxes",
												name: "mulu_config.mulu_mode",
												label: "泛目录路由",
												checkAll: true,
												optionType: "button",
												options: [
													{ label: "404页面", value: "404" },
													{ label: "非首页（所有页面）", value: "all_page" },
													{ label: "自定义路径", value: "custom_header" },
												]
											},
											{
												"type": "input-array",
												"name": "mulu_config.mulu_custom_header",
												"label": "自定义路径",
												"items": {
													"type": "input-text",
													"name": "/",
													"label": "/",
													"unique": true,
												},
												"addButtonText": "泛目录路径",
												"minItems": 0,
											},
											{
												"type": "input-array",
												"name": "mulu_config.mulu_template",
												"label": "泛目录模板",
												"items": {
													"type": "input-text",
													"name": "",
													"label": "",
													"unique": true,
												},
												"addButtonText": "泛目录模板",
												"minItems": 0,
											},
											{
												"type": "input-array",
												"name": "mulu_config.mulu_keywords_file",
												"label": "关键词库",
												"items": {
													"type": "input-text",
													"name": "词库路径",
													"label": "词库路径",
													"unique": true,
												},
												"addButtonText": "关键词库",
												"minItems": 0,
											},
											{
												"type": "static-datetime",
												"name": "updated_at",
												"label": "更新于"
											},
											{
												"type": "static-datetime",
												"name": "created_at",
												"label": "创建于"
											}
										]
									}
								}
							},
							{
								"type": "button",
								"icon": "fa fa-eraser text-danger",
								"actionType": "ajax",
								"tooltipPlacement": "top",
								"tooltip": "清空缓存",
								"confirmText": "确认清空【${website_info.domain}】 所有缓存数据？",
								"api": "delete:/_api_/cache/delete?domains=$website_info.domain",
								"reload": "none"
							},
							{
								"type": "button",
								"icon": "fa fa-trash text-danger",
								"actionType": "ajax",
								"tooltipPlacement": "top",
								// "tooltip": "删除",
								"confirmText": "确认删除【${id}】${domain}",
								"api": "delete:/_api_/website/delete?files=$id"
							}
						],
						"toggled": true
					}
				]
			}
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
