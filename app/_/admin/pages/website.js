(function () {
	const response = {
		data: {
			"type": "page",
			"title": "网站管理",
			"toolbar": [

			],

			"body": {
				"type": "crud",
				"itemBadge": {
					"text": "${is_www ? '主站' : '泛站'}",
					// "variations": {
					// 	"true": "primary",
					// 	"false": "danger"
					// },
					"mode": "ribbon",
					"position": "top-left",
					"level": "${is_www ? 'info' : 'danger'}",
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
				"autoGenerateFilter": true,
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
							"title": "新建网站",
							"body": {
								"type": "form",
								"size": "lg",
								"name": "sample-edit-form",
								"api": "post:/_api_/website/insert",
								"reload": "crud-table", // 在提交后重新加载特定的组件
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
									{
										"type": "select",
										"name": "link_mapping",
										"label": "链接映射",
										// "required": true,
										"options": [
											{
												"label": "开启",
												"value": "true"
											},
											{
												"label": "关闭",
												"value": "false"
											}
										],
										"value": "false",  // 设置默认值为 zh
										"placeholder": "是否开启链接映射"
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
												"label": "目标站替换词",
												"value": "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'"
											}
										]
									},
									{
										"type": "alert",
										"body": "注意：替换词格式按照“先长后短”方式，如“hello world -> {关键词}”在上，“hello -> 你好”在下",
									},
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
										"type": "editor",
										"language": "yaml",
										"name": "replace_string",
										"label": "本站替换词",
										"value": "全局替换:\n  - '待替换字符串 -> 替换词'\n首页替换:\n  - '待替换字符串 -> 替换词'\n内页替换:\n  - '待替换字符串 -> 替换词'"
									}
								]
							}
						}
					}, {
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
								"reload": "crud-table", // 在提交后重新加载特定的组件
								"body": [
									{
										"type": "select",
										"name": "over_write",
										"label": "建站模式",
										// "required": true,
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
										// "required": true,
										"options": [
											{
												"label": "覆盖",
												"value": true
											},
											{
												"label": "存在则跳过",
												"value": false
											}
										],
										"value": false,
										"placeholder": "是否覆盖"
									},
									// 插入新的 service，用于加载 预建站文档 数据
									{
										"type": "service",
										"api": "/_api_/file/query?path=doc/website.txt",  // 动态加载 预建站文档
										"body": [
											{
												"type": "alert",
												"body": "格式：<域名>__<目标站>__<链接映射(true/false)>__<标题>__<关键词>__<描述>__<替换模式(0/1/2/3)>__<目标站替换词(可留空)>__<本站替换词(可留空)>",
											},
											{
												"type": "alert",
												"body": "例子：www.domain.com__en|www.target.com__true__网站标题__网站关键词__网站描述__1__关于我们----------{keyword}##########公司名称----------【关键词】__关于我们 -> {keyword} ; 公司名称 -> 【关键词】",
											},
											{
												"type": "editor",
												"language": "yaml",
												"name": "content",
												"label": "建站信息",
												"placeholder": "<域名>__<目标站>__<链接映射(true/false)>__<标题>__<关键词>__<描述>__<替换模式(0/1/2/3)>__<目标站替换词(可留空)>__<本站替换词(可留空)>",
												"value": ""
											},
											{
												"type": "alert",
												"body": "替换词 标准格式： 被替换词与替换词的间隔符为“ -> ”，多组分隔符为“ ; ”，如：关于我们 -> {keyword} ; 公司名称 -> 【关键词】",
											},
											{
												"type": "alert",
												"body": "替换词 兼容格式(为兼容之前版本格式)： 被替换词与替换词的间隔符为“----------”，多组分隔符为“##########”，如：关于我们----------{keyword}##########公司名称----------【关键词】",
											},
											
											
										]
									}
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
						"name": "index",
						"label": "序号",
						// "fixed": "left",
						// "sortable": true,  // 启用排序功能
					},
					{
						"name": "id",
						"label": "文件路径",
						"searchable": {
							"type": "textarea",
							"name": "search_term",
							"label": "🔍搜索",
							"clearable": true,
							"maxLength": 10000,
							"showCounter": true,
						},
						// "fixed": "left",
						// "sortable": true,  // 启用排序功能
						"visible": false
					},
					{
						"type": "static-mapping",
						"name": "is_www",
						"label": "站点类型",
						"map": {
							"true": "主站",
							"false": "泛站"
						},
						"visible": false
						// "sortable": true,
						// "searchable": {
						// 	"type": "select",
						// 	"name": "is_www",
						// 	"label": "站点类型",
						// 	"options": [
						// 		{
						// 			"label": "主站+泛站",
						// 			"value": 0
						// 		},
						// 		{
						// 			"label": "主站",
						// 			"value": 1
						// 		},
						// 		{
						// 			"label": "泛站",
						// 			"value": 2
						// 		}
						// 	],
						// 	"value": 0,  // 默认值设置为 "主站+泛站"
						// 	"placeholder": "选择站点类型"
						// }
					},
					{
						"type": "tpl",
						"tpl": "<a href='http://${domain}' target='_blank' class='link-style'>${domain}</a>",
						"name": "domain",
						"label": "域名",
						"fixed": "left",
						"copyable": true
						// "searchable": true,
						// "sortable": true
					},
					{
						"name": "lang",
						"label": "语言",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"name": "root_domain",
						"label": "根域名",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"type": "tpl",
						"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${target}</a>",
						"name": "target",
						"label": "目标站",
						// "sortable": true,
						"searchable": true,
						"onEvent": {
							"click": {
								"actions": [
									{
										"actionType": "custom",
										"script": "const parts = event.data.target.split('|'); if(parts.length > 1) { const linkTarget = parts[1]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
									}
								]
							}
						}
					},
					{
						"name": "title",
						"label": "网站标题",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"name": "keywords",
						"label": "关键词",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					{
						"name": "description",
						"label": "描述",
					},
					{
						"name": "replace_string",
						"label": "本站替换词",
						"hidden": true  // 隐藏该字段
					},
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
						"width": 155,
						"buttons": [
							{
								"type": "button",
								"icon": "fa fa-refresh text-danger",
								"actionType": "ajax",
								"tooltip": "换目标站",
								"confirmText": "确认随机更换【${id}】${domain} 目标站: ${target}",
								"api": "get:/_api_/website/random_target?id=$id"
							},
							{
								"type": "button",
								"icon": "fa fa-times text-danger",
								"actionType": "ajax",
								"tooltip": "删除目标站",
								"confirmText": "确认删除 目标站库中的: ${target}",
								"api": "delete:/_api_/file/config/target.txt?line=$target",
								"reload": "none"
							},
							{
								"type": "button",
								"icon": "fa fa-pencil",
								"tooltip": "编辑",
								"actionType": "drawer",
								"drawer": {
									"resizable": true,
									"size": "lg",
									"title": "编辑",
									"body": {
										"type": "form",
										"name": "sample-edit-form",
										"api": "put:/_api_/website/update?file=$id",
										"reload": "crud-table", // 在提交后重新加载特定的组件
										"body": [
											{
												"type": "static",
												"name": "domain",
												"label": "域名",
											},
											{
												"type": "static",
												"name": "lang",
												"label": "语言",
											},
											{
												"type": "static",
												"name": "root_domain",
												"label": "根域名",
											},
											{
												"type": "static-mapping",
												"name": "is_www",
												"label": "站点类型",
												"map": {
													"true": "主站",
													"false": "泛站"
												}
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
												// "value": "false",  // 设置默认值为 zh
												// "placeholder": "是否开启链接映射"
											},
											{
												"type": "input-text",
												"name": "title",
												"label": "网站标题",
												"required": true
											},
											{
												"type": "input-text",
												"name": "keywords",
												"label": "关键词"
											},
											{
												"type": "textarea",
												"name": "description",
												"label": "描述"
											},
											{
												"type": "input-text",
												"name": "target",
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
												"api": "/_api_/replace/query?domain=$target",  // 动态加载 target_replace 数据的 API
												"body": [
													{
														"type": "editor",
														"language": "yaml",
														"name": "target_replace",
														"label": "目标站替换词",
														"value": "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'"
													}
												]
											},
											{
												"type": "alert",
												"body": "注意：替换词格式按照“先长后短”方式，如“hello world -> {关键词}”在上，“hello -> 你好”在下",
											},
											{
												"type": "select",
												"name": "replace_mode",
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
												"type": "editor",
												"language": "yaml",
												"name": "replace_string",
												"label": "本站替换词"
											},
											{
												"type": "static-datetime",
												"name": "updated_at",
												"label": "更新于"
											}
										]
									}
								}
							},
							{
								"type": "button",
								"icon": "fa fa-eraser text-danger",
								"actionType": "ajax",
								"tooltip": "清空缓存",
								"confirmText": "确认清空【${id}】${domain} 所有页面缓存",
								"api": "delete:/_api_/cache/delete?domain=$domain",
								"reload": "none"
							},
							{
								"type": "button",
								"icon": "fa fa-trash text-danger",
								"actionType": "ajax",
								"tooltip": "删除",
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
