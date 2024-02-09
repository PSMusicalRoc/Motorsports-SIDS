console.log(window.location.origin);
const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");

websocket.onopen = function() {
};

websocket.onmessage = function(ev) {
    try {
        let jsObj = JSON.parse(ev.data);
        
        if (jsObj.msgtype == "null") {
            console.log("Null message");
        } else if (jsObj.msgtype == "message") {
            console.log("Server message: " + jsObj.message);
        } else if (jsObj.msgtype == "in_shop_add") {
            
        }
    } catch (e) {
        console.log(e);
    }
}

function press_da_button() {
    // websocket.send("pressed_a_button");
    websocket.send("send_a_json");
}