(function () {
    const response = {
        data: {
            type: "page",
            // title: "设置",
            body: [
                {
                    type: "form",
                    title: " ",
                    name: "set_form",
                    mode: "horizontal",
                    horizontal: {
                        "leftFixed": true
                        // "left": 2,
                        // // "right": 100,
                        // "offset": 2
                    },
                    labelWidth: 200,
                    api: {
                        "method": "put",
                        "url": "/_api_/config",
                        // "requestAdaptor": function (api) {
                        // 	if (api.data && typeof api.data === 'object') {
                        // 		Object.keys(api.data).forEach(function (key) {
                        // 			if (typeof api.data[key] === 'string') {
                        // 				api.data[key] = api.data[key].replace(/<script/g, '<3cript');
                        // 			}
                        // 		});
                        // 	}
                        // 	return api;
                        // }
                    },
                    initApi: "/_api_/config", // 从后端获取初始数据
                    actions: [
                        // {
                        // 	"type": "tpl",
                        // 	"tpl": "<a href='https://t.me/MirrorElf' target='_blank'>续费</a>"
                        // },
                        {
                            "type": "service",
                            "api": "/_api/version?mode=0",  // 动态加载
                            "body": [
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
                                    "label": "保存设置"  // 修改提交按钮文本
                                },
                            ]
                        },
                    ],
                    reload: "set_form",
                    body: [
                        {
                            "label": "${authorization_end_info}",
                            "type": "tag",
                            "color": "error",
                            "displayMode": "status",
                            "visibleOn": "this.authorization_end_info",
                            "style": {
                                "display": "inline-block",
                                "width": "100%",
                                "text-align": "center"
                            }
                        },
                        // {
                        //     "type": "tag",
                        //     "label": "error",
                        //     "displayMode": "normal",
                        //     "color": "error"
                        // },
                        {
                            type: "anchor-nav",
                            direction: "horizontal",
                            style: {
                                height: "70vh",
                            },
                            links: [
                                {
                                    title: "网站信息",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "网站信息",
                                            body: [
                                                {
                                                    name: "ProgramInfo.program_name",
                                                    type: "input-text",
                                                    label: "程序名称"
                                                },
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            name: "ProgramInfo.login_account",
                                                            type: "input-text",
                                                            label: "登录账号",
                                                            required: true,
                                                        },
                                                        {
                                                            name: "ProgramInfo.login_password",
                                                            type: "input-password",
                                                            label: "登录密码",
                                                            required: true,
                                                            hint: "默认账号密码admin，请及时修改"
                                                        },

                                                    ]
                                                },
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            name: "ProgramInfo.authorization_code",
                                                            type: "input-text",
                                                            label: "授权码${WebsiteSettings.auto_https_certificate ? '✅' : '⛔'}",
                                                            required: true,
                                                            "status": "pending",
                                                            desc: "${authorization_info} 新服务器赠送1天时间的授权码，发送“右下角-设备机器码”至 https://t.me/MirrorElf 领取免费授权码"
                                                        },
                                                    ]
                                                },
                                                {
                                                    name: "ProgramInfo.amazon_s3_api",
                                                    type: "input-text",
                                                    label: "amazon_s3_api",
                                                    disabled: true,  // 设置为只读
                                                    desc: "amazon_s3_api分布式文件系统接口信息，如需更改请于服务器文件中修改。(重启程序生效)"
                                                },
                                                // {
                                                //     name: "ProgramInfo.safeline_token",
                                                //     type: "input-text",
                                                //     label: "雷池Token",
                                                //     desc: "与防火墙通信，展示网站数据、自动https证书，请务必正确填写"
                                                // }
                                            ]
                                        }
                                    ]
                                },
                                {
                                    title: "建站设置",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "建站设置",
                                            body: [
                                                {
                                                    name: "WebsiteSettings.auto_site_building",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "自动建站"
                                                },
                                                {
                                                    name: "WebsiteSettings.language",
                                                    type: "radios",
                                                    label: "自动建站语言",
                                                    value: "zh",
                                                    options: [
                                                        { label: "中文", value: "zh" },
                                                        { label: "英文", value: "en" },
                                                        { label: "葡萄牙文", value: "pt" }
                                                    ]
                                                },
                                                // {
                                                //     name: "WebsiteSettings.auto_https_certificate",
                                                //     type: "switch",
                                                //     onText: "开启",
                                                //     offText: "关闭",
                                                //     label: "自动https"
                                                // },
                                                {
                                                    name: "WebsiteSettings.pan_site_auto_site_building",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "泛站自动建站"
                                                },
                                                {
                                                    name: "WebsiteSettings.pan_site_crawler_target",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "泛站爬取目标"
                                                },
                                                {
                                                    name: "WebsiteSettings.link_mapping",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "链接映射"
                                                },
                                                {
                                                    name: "WebsiteSettings.homepage_update_time",
                                                    type: "input-number",
                                                    label: "首页更新时间",
                                                    required: true,
                                                    desc: "单位：天 填写0则永不更新首页"
                                                },
                                                {
                                                    name: "WebsiteSettings.target_static_save",
                                                    type: "checkboxes",
                                                    label: "静态资源本地化",
                                                    checkAll: true,
                                                    optionType: "button",
                                                    options: [
                                                        {
                                                            "label": "文本资源",
                                                            "children": [
                                                                { "label": "js", "value": ".js" },
                                                                { "label": "css", "value": ".css" },
                                                                { "label": "txt", "value": ".txt" },
                                                                { "label": "json", "value": ".json" },
                                                                { "label": "pdf", "value": ".pdf" },
                                                                { "label": "xml", "value": ".xml" }
                                                            ]
                                                        },
                                                        {
                                                            "label": "图片资源",
                                                            "children": [
                                                                { "label": "jpg", "value": ".jpg" },
                                                                { "label": "png", "value": ".png" },
                                                                { "label": "gif", "value": ".gif" },
                                                                { "label": "svg", "value": ".svg" },
                                                                { "label": "webp", "value": ".webp" },
                                                                { "label": "ico", "value": ".ico" },
                                                                { "label": "avif", "value": ".avif" }
                                                            ]
                                                        },
                                                        {
                                                            "label": "字体资源",
                                                            "children": [
                                                                { "label": "woff2", "value": ".woff2" },
                                                                { "label": "woff", "value": ".woff" },
                                                                { "label": "ttf", "value": ".ttf" },
                                                                { "label": "otf", "value": ".otf" }
                                                            ]
                                                        },
                                                        {
                                                            "label": "媒体资源",
                                                            "children": [
                                                                { "label": "mp3", "value": ".mp3" },
                                                                { "label": "mp4", "value": ".mp4" },
                                                                { "label": "webm", "value": ".webm" },
                                                                { "label": "wav", "value": ".wav" }
                                                            ]
                                                        },
                                                        {
                                                            "label": "压缩包",
                                                            "children": [
                                                                { "label": "zip", "value": ".zip" },
                                                                { "label": "rar", "value": ".rar" },
                                                                { "label": "7z", "value": ".7z" }
                                                            ]
                                                        },
                                                        {
                                                            "label": "开发资源",
                                                            "children": [
                                                                { "label": "ts", "value": ".ts" },
                                                                { "label": "scss", "value": ".scss" },
                                                                { "label": "map", "value": ".map" },
                                                                { "label": "wasm", "value": ".wasm" }
                                                            ]
                                                        }
                                                    ]
                                                },
                                            ]
                                        }
                                    ]
                                },
                                // {
                                //     title: "目录功能",
                                //     body: [
                                //         {
                                //             type: "fieldSet",
                                //             collapsable: true,
                                //             title: "目录功能",
                                //             body: [
                                //                 {
                                //                     name: "DirectoryFunctions.template_mode",
                                //                     type: "radios",
                                //                     label: "模板模式",
                                //                     value: "关闭",
                                //                     options: [
                                //                         { label: "关闭: 不调用模板", value: "关闭" },
                                //                         { label: "404: 目标站404时返回模板", value: "404" },
                                //                         { label: "泛目录: 所有内页调用模板，缓存页面", value: "泛目录" },
                                //                         { label: "蜘蛛池: 所有内页调用模板，不缓存页面", value: "蜘蛛池" }
                                //                     ]
                                //                 },
                                //                 {
                                //                     name: "DirectoryFunctions.tdk_format",
                                //                     type: "input-text",
                                //                     label: "TDK格式"
                                //                 },
                                //                 {
                                //                     name: "DirectoryFunctions.map_link_format",
                                //                     type: "input-text",
                                //                     label: "地图链接格式"
                                //                 },
                                //                 {
                                //                     name: "DirectoryFunctions.map_link_count",
                                //                     type: "input-number",
                                //                     label: "地图链接数量",
                                //                     required: true
                                //                 }
                                //             ]
                                //         }
                                //     ]
                                // },
                                {
                                    title: "SEO功能",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "SEO功能",
                                            body: [
                                                {
                                                    name: "SEOFunctions.external_link_strategy",
                                                    type: "checkboxes",
                                                    label: "外链替换策略",
                                                    checkAll: true,
                                                    optionType: "button",
                                                    options: [
                                                        { label: "当前域名·内链", value: "1" },
                                                        { label: "主站·内链", value: "2" },
                                                        { label: "泛站·首页", value: "3" },
                                                        { label: "泛站·内链", value: "4" },
                                                        { label: "【全服】主站·首页", value: "5" },
                                                        { label: "【全服】主站·内链", value: "6" },
                                                        { label: "【全服】泛站·首页", value: "7" },
                                                        { label: "【全服】泛站·内链", value: "8" },
                                                    ]
                                                },
                                                {
                                                    name: "SEOFunctions.random_class_attributes",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "随机class属性"
                                                },
                                                {
                                                    name: "SEOFunctions.random_meta_and_link",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "随机meta&link"
                                                },
                                            ]
                                        }
                                    ]
                                },
                                {
                                    title: "访问策略",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "访问策略",
                                            body: [
                                                {
                                                    name: "AccessPolicy.forced_domain_binding",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "强制域名绑定"
                                                },
                                                {
                                                    name: "AccessPolicy.pan_site_referrer",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "泛站来路"
                                                },
                                                {
                                                    name: "AccessPolicy.ip_site_referrer",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "IP与非域名来路"
                                                },

                                                // {
                                                //     name: "AccessPolicy.ua_banlist",
                                                //     type: "textarea",
                                                //     label: "UA黑名单"
                                                // },
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            "type": "input-array",
                                                            "name": "AccessPolicy.ua_banlist",
                                                            "label": "UA黑名单",
                                                            "items": {
                                                                "type": "input-text",
                                                                "name": "ua",
                                                                "label": "ua",
                                                            },
                                                            "addButtonText": "添加 UA",
                                                            "minItems": 0,
                                                            "unique": true,
                                                            "validationErrors": {
                                                                "unique": "IP 地址不能重复"
                                                            }
                                                        },
                                                        {
                                                            "type": "input-array",
                                                            "name": "AccessPolicy.ip_banlist",
                                                            "label": "IP黑名单",
                                                            "items": {
                                                                "type": "input-text",
                                                                "name": "ip",
                                                                "label": "ip",
                                                                "maxLength": 15,
                                                            },
                                                            "addButtonText": "添加 IP",
                                                            "minItems": 0,
                                                            "unique": true,
                                                            "validationErrors": {
                                                                "unique": "IP 地址不能重复"
                                                            }
                                                        }]
                                                },
                                            ]
                                        }
                                    ]
                                },
                                {
                                    title: "广告策略",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "广告策略",
                                            body: [
                                                {
                                                    name: "AdPolicy.search_referrer_jump_ad",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "搜索来路跳广告"
                                                },
                                                {
                                                    name: "AdPolicy.regular_ua_jump_ad",
                                                    type: "switch",
                                                    onText: "开启",
                                                    offText: "关闭",
                                                    label: "普通用户跳广告"
                                                },
                                                {
                                                    name: "AdPolicy.ad_url",
                                                    type: "input-text",
                                                    label: "广告URL"
                                                }
                                            ]
                                        }
                                    ]
                                },
                                {
                                    title: "全局JS代码",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "全局JS代码",
                                            body: [
                                                {
                                                    "type": "input-array",
                                                    "name": "GlobalCodeInsertion.filter_ip",
                                                    "label": "过滤地址",
                                                    "items": {
                                                        "type": "input-text",
                                                        "name": "ip",
                                                        "label": "ip",
                                                        "maxLength": 15,
                                                    }
                                                    ,
                                                    "addButtonText": "添加 IP",
                                                    "minItems": 0,
                                                    "unique": true,
                                                    "validationErrors": {
                                                        "unique": "IP 地址不能重复"
                                                    }
                                                },
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            name: "GlobalCodeInsertion.head_header",
                                                            type: "textarea",
                                                            label: "head头部"
                                                        },
                                                        {
                                                            name: "GlobalCodeInsertion.head_footer",
                                                            type: "textarea",
                                                            label: "head尾部"
                                                        }]
                                                },
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            name: "GlobalCodeInsertion.body_header",
                                                            type: "textarea",
                                                            label: "body头部"
                                                        },
                                                        {
                                                            name: "GlobalCodeInsertion.body_footer",
                                                            type: "textarea",
                                                            label: "body尾部"
                                                        }]
                                                }
                                            ]
                                        }
                                    ]
                                },
                                {
                                    title: "蜘蛛策略",
                                    body: [
                                        {
                                            type: "fieldSet",
                                            collapsable: true,
                                            title: "蜘蛛策略",
                                            body: [
                                                {
                                                    "type": "group",
                                                    "body": [
                                                        {
                                                            name: "SpiderPolicy.google_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "谷歌蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.google_img_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "谷歌图片蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.baidu_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "百度蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.sogou_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "搜狗蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.yisou_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "神马蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.byte_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "头条蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.bing_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "必应蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.so_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "360蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.quark_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "夸克蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.yahoo_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "雅虎蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.other_spider",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "其它蜘蛛"
                                                        },
                                                        {
                                                            name: "SpiderPolicy.user",
                                                            type: "switch",
                                                            onText: "允许",
                                                            offText: "禁止",
                                                            label: "普通用户"
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
                }
            ]
        },
        status: 0
    };

    window.jsonpCallback && window.jsonpCallback(response);
})();