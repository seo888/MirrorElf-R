(function () {
	const response = {
		data: {
			"type": "grid",
			"title": "缓存管理",
			"columns": [
				{
					"type": "grid",
					"columns": [
						{
							"md": 2,  // 左侧占 3 份宽度（25%）
							"body": {
								"type": "crud",
								"itemBadge": {
									"text": "${is_www? '主站' : '泛站'}",
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
									"level": "${is_www? 'info' : 'danger'}",
									// "visibleOn": "this.is_www"
								},
								// "filterTogglable": true,
								"autoGenerateFilter": true,
								"filter": {
									"wrapWithPanel": false,
									"title": "搜索",
									"body": [
										{
											"type": "select",
											"name": "is_www",
											"label": "",
											"options": [
												{
													"label": "主",
													"value": "true"
												},
												{
													"label": "泛",
													"value": ""
												},
											],
											"value": "true",  // 默认值设置为 "主站+泛站"
											"placeholder": "选择站点类型"
										},
										{
											"type": "input-text",
											"name": "domain",
											"prefix": "🔍",
											"addOn": {
												"type": "submit",  // 显式添加搜索按钮
												"label": "搜索",
												"level": "primary",
											},
											"clearable": true
										},


									],
								},
								"headerToolbar": [
									"bulkActions",
									{
										"type": "tpl",
										// "tpl": "主站缓存: 17 | 泛站缓存: 3 | 共: 20",
										"tpl": "共: ${count}个站点",
										"className": "v-middle"
									},],
								"itemActions": [
									{
										"type": "button",
										"icon": "fa fa-trash text-danger",
										"tooltip": "清空",
										"actionType": "ajax",
										"confirmText": "确认清空【${target_lib}】${domain}的所有数据？",
										"api": "delete:/_api_/target/delete?bucket=$target_lib&domain=$domain",
									},
								],
								"api": {
									"url": "/_api_/cache/domains",
								},
								"itemAction": {
									"actionType": "reload",
									"target": "detailCRUD?domain=${domain}&page=1"
								},
								"columns": [
									{
										"name": "index",
										"width": 50,
										"label": "序号"
									},
									{
										"name": "domain",
										"label": "域名",
										"type": "text",
									},
									// {
									// 	"type": "static-mapping",
									// 	"name": "is_www",
									// 	"label": "站点类型",
									// 	"visible": false,
									// 	"map": {
									// 		"true": "<span class='label label-success'>主站</span>",
									// 		"false": "<span class='label label-danger'>泛站</span>",
									// 	},
									// 	"searchable": {
									// 		"type": "select",
									// 		"name": "is_www",
									// 		"label": "站点类型",
									// 		"options": [
									// 			{
									// 				"label": "主站+泛站",
									// 				"value": ""
									// 			},
									// 			{
									// 				"label": "主站",
									// 				"value": "true"
									// 			}
									// 		],
									// 		"value": "true",  // 默认值设置为 "主站+泛站"
									// 		"placeholder": "选择站点类型"
									// 	}
									// },
								]
							}
						},
						{
							"md": 10,  // 右侧占 9 份宽度（75%）
							"body": {
								"type": "crud",
								"name": "detailCRUD",
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
								"api": "/_api_/cache/query",
								"deferApi": "/_api_/cache/query?domain=${domain}",
								"perPageAvailable": [
									10,
									20,
									100,
									500,
								],
								"perPage": 10,
								"keepItemSelectionOnPageChange": true,
								"autoFillHeight": true,
								"labelTpl": "【${id}】",
								// "autoGenerateFilter": true,
								"filter": {
									// "mode": "inline",
									// "debug": true,
									"width": "600px",
									"wrapWithPanel": false,
									"title": "搜索",
									"body": [
										{
											"type": "group",  // 使用 group 组件
											"body": [
												{
													"type": "input-text",
													"name": "search_term",
													"prefix": "${domain} 🔍",
													addOn: {
														"type": "submit",  // 显式添加搜索按钮
														"label": "搜索",
														"level": "primary",
													},
													"clearable": true
												}
											]
										}
									],
								},
								// "autoGenerateFilter": {
								// 	// "columnsNum": 2,
								// 	"showBtnToolbar": false
								// },
								"bulkActions": [
									{
										"label": "批量删除",
										"level": "danger",
										"actionType": "ajax",
										"api": "delete:/_api_/target/delete?bucket=$target_lib&files=${ids|raw}",
										"confirmText": "确认批量删除【${target_lib}】${ids|raw}（注意：操作不可逆，请谨慎操作）",
										"onEvent": {
											"click": {
												"actions": [
													{
														"actionType": "setValue",
														"componentId": "crud-table", // 替换为你的 CRUD 组件 ID
														"args": {
															"value": {
																"rows": "${rows.map(row => row.id === event.data.current.id ? { ...row, children: [] } : row)}"
															}
														}
													}
												]
											}
										}
									}
								],
								"filterTogglable": true,
								"headerToolbar": [
									"bulkActions",
									{
										"type": "tpl",
										"tpl": "【<a href='http://${domain}' target='_blank' class='link-style'>${domain}</a>】 | URL: ${count}条",
										"className": "v-middle"
									},
									{
										"type": "button",
										"label": "",
										"icon": "fa fa-sync",
										"onEvent": {
											"click": {
												"actions": [
													{
														"actionType": "setValue",
														"componentId": "crud-table",  // 替换为你的表格组件 ID
														"args": {
															"value": {
																"rows": []  // 将数据设置为空数组
															}
														}
													},
													{
														"actionType": "reload",
														"componentId": "crud-table",  // 替换为你的表格组件 ID
													}
												]
											}
										}
									},
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
									}
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
										"name": "id",
										"label": "ID",
									},
									{
										"name": "id",
										"label": "文件路径",
										// "searchable": {
										// 	"type": "input-text",
										// 	"name": "search_term",
										// 	"label": "🔍搜索",
										// },
										"visible": false
									},
									{
										"type": "tpl",
										"tpl": "<a href='${url}' target='_blank' class='link-style'>${url}</a>",
										"name": "url",
										"label": "URL",
									},
									{
										"type": "tpl",
										"tpl": "<a href='http://${domain}${uri}' target='_blank' class='link-style'>${uri}</a>",
										"name": "uri",
										"label": "真实路径",
									},
									{
										name: "title",
										label: "标题",
									}, {
										name: "domain",
										label: "域名",
										"visible": false
									},
									{
										"type": "static-mapping",
										"name": "page_type",
										"fixed": "right",
										"label": "页面类型",
										"map": {
											"缓存": "<span class='label label-success'>缓存</span>",
											"映射": "<span class='label label-warning'>映射</span>",
											"目录": "<span class='label label-info'>目录</span>",
											"静态": "<span class='label label-danger'>静态</span>",
										},

									},
									// {
									// 	"type": "tpl",
									// 	"tpl": "<a href='http://${target}' target='_blank' class='link-style'>${target}</a>",
									// 	"name": "target",
									// 	"label": "目标路径",
									// },
									{
										"type": "tpl",
										"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${target}</a>",
										"name": "target",
										"label": "目标站",
										// "sortable": true,
										// "searchable": true,
										"onEvent": {
											"click": {
												"actions": [
													{
														"actionType": "custom",
														"script": "const parts = event.data.target.split('://'); if(parts.length > 1) { const linkTarget = parts[1]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
													}
												]
											}
										}
									},
									{
										"type": "datetime",  // 显示为日期时间类型
										"name": "updated_at",
										"label": "更新于",
										"fixed": "right",
										"sortable": true,  // 启用排序功能
									},
									{
										"type": "operation",
										"fixed": "right",
										"buttons": [
											{
												"icon": "fa fa-trash text-danger",
												"actionType": "ajax",
												// "tooltipPlacement": "right",
												// "tooltip": "删除",
												"confirmText": "确认删除【${target_lib}】${id}",
												"api": "delete:/_api_/target/delete?bucket=$target_lib&files=$id",
											}
										]
									}
								]
							}
						}
					]
				}
			]
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();

