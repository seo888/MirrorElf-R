(function () {
    const baseUrl = window.location.origin.replace('http://', 'https://'); // 获取当前页面的域名
    const targetUrl = baseUrl + "/_tag.html";
    const response = {
        data: {
            "type": "page",
            "body": {
                "type": "iframe",
                "src": targetUrl
            }
        },
        status: 0
    }

    window.jsonpCallback && window.jsonpCallback(response);
})();
