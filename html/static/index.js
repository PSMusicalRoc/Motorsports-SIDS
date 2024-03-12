const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");
let globalTimeout = undefined;

websocket.onopen = function() {
    this.send("get_in_shop");
};

websocket.onmessage = function(ev) {
    try {
        let jsObj = JSON.parse(ev.data);
        
        if (jsObj.msgtype == "null") {
            console.log("Null message");
        } else if (jsObj.msgtype == "in_shop_refresh") {
            let people = JSON.parse(jsObj.message);
            createInShopTable(people);
        } else if (jsObj.msgtype == "timestamps_refresh") {
            let people = JSON.parse(jsObj.message);
            createAllTimeTable(people);
        } else if (jsObj.msgtype == "parsing") {
            clearTimeout(globalTimeout);
            let text = document.getElementById("status-text");
            text.innerHTML = "Parsing Keycard...";
            text.className = "status-parse";
        } else if (jsObj.msgtype == "unknown_person") {
            clearTimeout(globalTimeout);
            let text = document.getElementById("status-text");
            text.innerHTML = "You are not registered in the RM database yet. Please speak with the Safety Advisor currently on-site to fix this.";
            text.className = "status-error";
            globalTimeout = setTimeout(() => {
                let text = document.getElementById("status-text");
                text.innerHTML = "Awaiting input!";
                text.className = "status-parse";
            }, 10000);
        } else if (jsObj.msgtype == "rfid_success") {
            clearTimeout(globalTimeout);
            let text = document.getElementById("status-text");
            text.innerHTML = "Verified!";
            text.className = "status-good";
            globalTimeout = setTimeout(() => {
                let text = document.getElementById("status-text");
                text.innerHTML = "Awaiting input!";
                text.className = "status-parse";
            }, 3000);
        }
    } catch (e) {
        console.log(e);
    }
}

function createInShopTable(people) {
    const tableroot = document.getElementById("currently-in-table");
    tableroot.innerHTML = "";

    let headerrow = document.createElement("tr");
    let nameheader = document.createElement("th");
    nameheader.innerHTML = "Name";
    headerrow.appendChild(nameheader);

    let idheader = document.createElement("th");
    idheader.innerHTML = "RCSID";
    headerrow.appendChild(idheader);

    let timeheader = document.createElement("th");
    timeheader.innerHTML = "Time In";
    headerrow.appendChild(timeheader);

    tableroot.appendChild(headerrow);
    
    
    for (var i = 0; i < people.length; i++) {
        let newrow = document.createElement("tr");
        let name = document.createElement("td");
        name.innerHTML = people[i].firstname + " " + people[i].lastname;
        newrow.appendChild(name);

        let rcsid = document.createElement("td");
        rcsid.innerHTML = people[i].rcsid;
        newrow.appendChild(rcsid);

        let timestamp = document.createElement("td");
        timestamp.innerHTML = people[i].timestamp;
        newrow.appendChild(timestamp);
        
        tableroot.appendChild(newrow);
    }

    websocket.send("get_all_timestamps");
}

function createAllTimeTable(people) {
    const tableroot = document.getElementById("all-timestamps-table");
    tableroot.innerHTML = "";

    let headerrow = document.createElement("tr");
    let nameheader = document.createElement("th");
    nameheader.innerHTML = "Name";
    headerrow.appendChild(nameheader);

    let idheader = document.createElement("th");
    idheader.innerHTML = "RCSID";
    headerrow.appendChild(idheader);

    let enteringheader = document.createElement("th");
    enteringheader.innerHTML = "Entering?";
    headerrow.appendChild(enteringheader);

    let timeheader = document.createElement("th");
    timeheader.innerHTML = "Timestamp";
    headerrow.appendChild(timeheader);

    tableroot.appendChild(headerrow);
    
    
    for (var i = 0; i < people.length; i++) {
        let newrow = document.createElement("tr");
        let name = document.createElement("td");
        name.innerHTML = people[i].firstname + " " + people[i].lastname;
        newrow.appendChild(name);

        let rcsid = document.createElement("td");
        rcsid.innerHTML = people[i].rcsid;
        newrow.appendChild(rcsid);

        let entering = document.createElement("td");
        entering.innerHTML = people[i].entering ? "In" : "Out";
        newrow.appendChild(entering);

        let timestamp = document.createElement("td");
        timestamp.innerHTML = people[i].timestamp;
        newrow.appendChild(timestamp);
        
        tableroot.appendChild(newrow);
    }
}