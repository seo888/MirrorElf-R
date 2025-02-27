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
				"api": "/_api_/target/query",
				"perPageAvailable": [
					10,
					20,
					100,
					500,
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
						"api": "delete:/_api_/target/delete?bucket=$target_lib&files=${ids|raw}",
						"confirmText": "确认批量删除【${target_lib}】${ids|raw}（注意：操作不可逆，请谨慎操作）"
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
					// {
					// 	"type": "tpl",
					// 	"tpl": "<a href='javascript:void(0);' class='link-icon' target='_blank'>${url}</a>",
					// 	"name": "url",
					// 	"label": "URL",
					// 	"sortable": true,
					// 	"searchable": true,
					// 	"onEvent": {
					// 		"click": {
					// 			"actions": [
					// 				{
					// 					"actionType": "custom",
					// 					"script": "const parts = event.data.url.split('['); if(parts.length > 0) { const linkTarget = parts[0]; document.querySelector('.link-icon').setAttribute('href', 'http://' + linkTarget); window.open('http://' + linkTarget, '_blank'); }"
					// 				}
					// 			]
					// 		}
					// 	}
					// },
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
					{
						"name": "status_code",
						"label": "状态码",
						// "sortable": true,  // 启用排序功能
						// "searchable": true,
					},
					// {
					// 	"name": "content_type",
					// 	"label": "内容类型",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					// {
					// 	"name": "title",
					// 	"label": "标题",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					{
						"type": "tpl",
						"tpl": "<a href='http://${domain}' target='_blank' class='link-style'>${domain}</a>",
						"name": "domain",
						"label": "域名",
						"fixed": "left",
						// "searchable": true,
						// "sortable": true
					},
					// {
					// 	"name": "root_domain",
					// 	"label": "根域名",
					// 	"sortable": true,  // 启用排序功能
					// 	"searchable": true,
					// },
					// {
					// 	"type": "datetime",  // 显示为日期时间类型
					// 	"name": "created_at",
					// 	"label": "创建于",
					// 	"sortable": true,  // 启用排序功能
					// },
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
								"confirmText": "确认删除【${target_lib}】${id}",
								"api": "delete:/_api_/target/delete?bucket=$target_lib&files=$id",
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
