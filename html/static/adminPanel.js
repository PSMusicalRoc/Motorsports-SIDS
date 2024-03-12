const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");
let globalTimeout = undefined;

websocket.onopen = function() {
};

websocket.onmessage = function(ev) {
    try {
        let jsObj = JSON.parse(ev.data);
        
        if (jsObj.msgtype == "null") {
            console.log("Null message");
        }
    } catch (e) {
        console.log(e);
    }
}

function add_person() {

    let firstname = document.getElementsByName("firstname")[0].value;
    let lastname = document.getElementsByName("lastname")[0].value;
    let rcsid = document.getElementsByName("rcsid")[0].value;
    let isGood = document.getElementById("isGood").checked;

    let sendStr = "new_person " + firstname + " " + lastname + " "
        + rcsid + " " + isGood;
    websocket.send(sendStr);

    return false;
}