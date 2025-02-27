(function () {
	const response = {
		data: {
			"type": "page",
			"title": "缓存管理",
			"toolbar": [

			],

			"body": {
				"type": "crud",
				"id": "crud-table",
				"syncLocation": false,
				// "quickSaveApi": "/_api_/cache/update?id=${id}",  // 更新 API 地址
				// "draggable": true,
				"api": "/_api_/cache/query?is_mapping=false",
				"perPageAvailable": [
					10,
					20,
					50,
					500,
					1000
				],
				"perPage": 20,
				"keepItemSelectionOnPageChange": true,
				"autoFillHeight": true,
				"labelTpl": "【${id}】",
				"autoGenerateFilter": true,
				"bulkActions": [
					{
						"label": "批量删除",
						"level": "danger",
						"actionType": "ajax",
						"api": "delete:/_api_/cache/delete?ids=${ids|raw}",
						"confirmText": "确认批量删除【缓存】URL【${ids|raw}】（注意：操作不可逆，请谨慎操作）"
					}
				],
				"filterTogglable": true,
				"headerToolbar": [
					"bulkActions",
					{
						"type": "tpl",
						"tpl": "【缓存】站点数量: ${site_count} | URL: ${total_count}条",
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
						"name": "index",
						"label": "序号",
						"fixed": "left",
						// "sortable": true,  // 启用排序功能
					},
					{
						"name": "id",
						"label": "文件路径",
						"searchable": {
							"type": "input-text",
							"name": "search_term",
							"label": "🔍搜索",
						},
						"fixed": "left",
						// "sortable": true,  // 启用排序功能
					},
					{
						"type": "tpl",
						"tpl": "<a href='${url}' target='_blank' class='link-style'>${url}</a>",
						"name": "url",
						"label": "URL",
						"fixed": "left",
						// "searchable": true,
						// "sortable": true
					},
					// {
					// 	"name": "lang",
					// 	"label": "语言",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					// {
					// 	"type": "tpl",
					// 	"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${target}</a>",
					// 	"name": "target",
					// 	"label": "目标url",
					// 	"sortable": true,
					// 	"searchable": true,
					// 	"onEvent": {
					// 		"click": {
					// 			"actions": [
					// 				{
					// 					"actionType": "custom",
					// 					"script": "const parts = event.data.target.split('['); if(parts.length > 0) { const linkTarget = parts[0]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
					// 				}
					// 			]
					// 		}
					// 	}
					// },
					// {
					// 	"name": "title",
					// 	"label": "标题",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					// {
					// 	"name": "keywords",
					// 	"label": "关键词",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					// {
					// 	"name": "description",
					// 	"label": "描述",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					
					
					{
						"type": "static-mapping",
						"name": "is_mapping",
						"label": "状态",
						"map": {
							"true": "映射链接",
							"false": "正常"
						},
						"searchable": {
							"type": "select",
							"name": "is_mapping",
							"label": "状态",
							"options": [
								{
									"label": "正常",
									"value": "false"
								},
								{
									"label": "正常+映射链接",
									"value": ''
								},
								{
									"label": "映射链接",
									"value": "true"
								},
							],
							"value": '',  // 默认值设置为 "正常"
							"placeholder": "选择站点类型"
						}
					},
					{
						"name": "domain",
						"label": "域名",
					},
					{
						"type": "tpl",
						"tpl": "<a href='http://${domain}${mapping_url}' target='_blank' class='link-style'>${mapping_url}</a>",
						"name": "mapping_url",
						"label": "映射",
						"fixed": "left",
					},
					{
						"type": "datetime",  // 显示为日期时间类型
						"name": "updated_at",
						"label": "更新于",
						"sortable": true,  // 启用排序功能
					},
					{
						"type": "operation",
						"label": "操作",
						"width": 130,
						"buttons": [
							{
								"type": "button",
								"icon": "fa fa-broom text-danger",
								"actionType": "ajax",
								"tooltip": "清空域名所有缓存",
								"confirmText": "确认清空 根域名: ${root_domain} 泛域名: *.${root_domain} 所有页面缓存",
								"api": "delete:/_api_/cache/delete?root_domain=$root_domain",
							},
							{
								"icon": "fa fa-pencil",
								"tooltip": "编辑源码",
								"actionType": "drawer",
								"drawer": {
									"resizable": true,
									"size": "lg",
									"title": "编辑源码",
									"body": {
										"type": "form",
										"name": "sample-edit-form",
										"api": "/_api_/cache/update?id=$id",
										"reload": "crud-table", // 在提交后重新加载特定的组件
										"body": [
											{
												"type": "static",
												"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${url}</a>",
												"name": "url",
												"label": "URL",
												"sortable": true,
												"searchable": true,
												"onEvent": {
													"click": {
														"actions": [
															{
																"actionType": "custom",
																"script": "const parts = event.data.url.split('['); if(parts.length > 0) { const linkTarget = parts[0]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
															}
														]
													}
												}
											},
											{
												"type": "static",
												"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${target}</a>",
												"name": "target",
												"label": "目标站",
												"sortable": true,
												"searchable": true,
												"onEvent": {
													"click": {
														"actions": [
															{
																"actionType": "custom",
																"script": "const parts = event.data.url.split('['); if(parts.length > 0) { const linkTarget = parts[0]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
															}
														]
													}
												}
											},											
											{
												"type": "static",
												"name": "title",
												"label": "网站标题",
											},
											{
												"type": "static",
												"name": "keywords",
												"label": "关键词"
											},
											{
												"type": "static",
												"name": "description",
												"label": "描述"
											},
											{
												"type": "service",
												"api": "/_api_/cache/get_source?url=$url",  // 动态加载 target_replace 数据的 API
												"body": [
													{
														"type": "editor",
														"language": "html",
														"name": "source",
														"label": "网页源码",
													}
												]
											},
											{
												"type": "static",
												"name": "created_at",
												"label": "创建于"
											},
											{
												"type": "static",
												"name": "updated_at",
												"label": "更新于"
											}
										]
									}
								}
							},
							{
								"icon": "fa fa-eraser text-danger",
								"actionType": "ajax",
								"tooltip": "清空缓存",
								"confirmText": "确认清空 域名: ${domain} 所有页面缓存",
								"api": "delete:/_api_/cache/delete?domain=$domain",
							},
							{
								"icon": "fa fa-trash text-danger",
								"actionType": "ajax",
								"tooltip": "删除",
								"confirmText": "确认删除缓存【${id}】${url}",
								"api": "delete:/_api_/cache/delete?ids=$id"
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
