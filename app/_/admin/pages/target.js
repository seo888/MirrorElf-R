(function () {
	const response = {
		data: {
			"type": "page",
			"title": "目标管理",
			"toolbar": [

			],

			"body": {
				"type": "crud",
				"id": "crud-table",
				"syncLocation": false,
				// "quickSaveApi": "/_api_/target_cache/update?id=${id}",  // 更新 API 地址
				// "draggable": true,
				"api": "/_api_/target_cache/query",
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
						"api": "delete:/_api_/target_cache/delete?ids=${ids|raw}",
						"confirmText": "确认批量删除【目标】URL【${ids|raw}】（注意：操作不可逆，请谨慎操作）"
					}
				],
				"filterTogglable": true,
				"headerToolbar": [
					"bulkActions",
					{
						"type": "tpl",
						"tpl": "【目标】URL 共: ${total_count}条",
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
						"name": "id",
						"label": "ID",
						"searchable": {
							"type": "input-text",
							"name": "search_term",
							"label": "🔍模糊搜索",
						},
						"fixed": "left",
						"sortable": true,  // 启用排序功能
					},

					{
						"type": "tpl",
						"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${url}</a>",
						"name": "url",
						"label": "【目标】URL",
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
						"name": "lang",
						"label": "语言",
						"sortable": true,  // 启用排序功能
						"searchable": true,
					},
					{
						"name": "status_code",
						"label": "状态码",
						"sortable": true,  // 启用排序功能
						"searchable": true,
					},
					{
						"name": "content_type",
						"label": "内容类型",
						"sortable": true,  // 启用排序功能
						"searchable": true,
					},
					{
						"name": "title",
						"label": "标题",
						"sortable": true,  // 启用排序功能
						"searchable": true,
					},
					{
						"type": "tpl",
						"tpl": "<a href='http://${domain}' target='_blank' class='link-style'>${domain}</a>",
						"name": "domain",
						"label": "域名",
						"fixed": "left",
						"searchable": true,
						"sortable": true
					},
					{
						"name": "root_domain",
						"label": "根域名",
						"sortable": true,  // 启用排序功能
						"searchable": true,
					},
					{
						"type": "datetime",  // 显示为日期时间类型
						"name": "created_at",
						"label": "创建于",
						"sortable": true,  // 启用排序功能
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
						"width": 60,
						"buttons": [
							{
								"icon": "fa fa-trash text-danger",
								"actionType": "ajax",
								"tooltip": "删除",
								"confirmText": "确认删除【${id}】${url}",
								"api": "delete:/_api_/target_cache/delete?ids=$id",
							},
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
