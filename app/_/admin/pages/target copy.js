(function () {
	const response = {
		data: {
			"type": "page",
			"title": "目标管理",
			"toolbar": [

			],

			"body": {
				"type": "crud",
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
				"api": "/_api_/target/query",
				"deferApi": "/_api_/target/query?target_lib=${target_lib}&file=${id}",
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
				"autoGenerateFilter": true,
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
						"tpl": "【${target_lib}】站点数量: ${site_count} | URL: ${total_count}条",
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
						"name": "index",
						"label": "序号",
					},
					{
						"name": "id",
						"label": "文件路径",
						"searchable": {
							"type": "input-text",
							"name": "search_term",
							"label": "🔍搜索",
						},
						"visible": false
					},
					{
						"type": "static-mapping",
						"name": "target_lib",
						"label": "目标库",
						"map": {
							"target-zh": "中文",
							"target-en2zh": "英译中",
							"target-en": "英文",
							"target-zh2en": "中译英",
						},
						"sortable": true,
						"searchable": {
							"type": "select",
							"name": "target_lib",
							"label": "目标库",
							"options": [
								{
									"label": "中文",
									"value": "target-zh"
								},
								{
									"label": "英译中",
									"value": "target-en2zh"
								},
								{
									"label": "英文",
									"value": "target-en"
								},
								{
									"label": "中译英",
									"value": "target-zh2en"
								}
							],
							"value": "target-zh",  // 默认值设置为 "中文"
							"placeholder": "选择目标库"
						}
					},
					{
						"type": "tpl",
						"tpl": "<a href='${url}' target='_blank' class='link-style'>${url}</a>",
						"name": "url",
						"label": "URL",
					},
					{
						"type": "static-mapping",
						"name": "status_code",
						"label": "状态码",
						"map": {
							"200": "<span class='label label-success'>200</span>",
							"*": "<span class='label label-danger'>${status_code}</span>"
						}
					},
					{
						"type": "tpl",
						"tpl": "<a href='http://${domain}' target='_blank' class='link-style'>${domain}</a>",
						"name": "domain",
						"label": "域名",
					},
					{
						"type": "datetime",  // 显示为日期时间类型
						"name": "updated_at",
						"label": "更新于",
						"sortable": true,  // 启用排序功能
					},
						{
						"type": "operation",
						"fixed": "right",
						"buttons": [
							{
								"icon": "fa fa-trash text-danger",
								"actionType": "ajax",
								"tooltip": "删除",
								"confirmText": "确认删除【${target_lib}】${id}",
								"api": "delete:/_api_/target/delete?bucket=$target_lib&files=$id",

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
						]
					}
				]
			}
		},
		status: 0
	}

	window.jsonpCallback && window.jsonpCallback(response);
})();
