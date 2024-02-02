console.log(window.location.origin);
const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");

websocket.onopen = function() {
    console.log("Opened!");
    this.send("hello");
};

websocket.onmessage = function(ev) {
    console.log(ev.data);
}

function press_da_button() {
    websocket.send("pressed_a_button");
}