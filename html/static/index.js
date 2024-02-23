console.log(window.location.origin);
const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");

websocket.onopen = function() {
    this.send("get_in_shop");
};

websocket.onmessage = function(ev) {
    try {
        let jsObj = JSON.parse(ev.data);

        /// @TODO REMOVE THIS
        console.log(jsObj);
        
        if (jsObj.msgtype == "null") {
            console.log("Null message");
        } else if (jsObj.msgtype == "message") {
            console.log("Server message: " + jsObj.message);
        } else if (jsObj.msgtype == "in_shop_refresh") {
            /// @TODO REMOVE THIS
            console.log(jsObj);
            let people = JSON.parse(jsObj.message);
            createInShopTable(people);
        } else if (jsObj.msgtype == "timestamps_refresh") {
            /// @TODO REMOVE THIS
            console.log(jsObj);
            let people = JSON.parse(jsObj.message);
            createAllTimeTable(people);
        }
    } catch (e) {
        console.log(e);
    }
}

function addperson() {
    let id = document.getElementById("input-person").value;
    websocket.send("add_to_shop " + id);
    document.getElementById("input-person").value = "";
}

function removeperson() {
    let id = document.getElementById("input-person").value;
    websocket.send("remove_from_shop " + id);
    document.getElementById("input-person").value = "";
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

        console.log(people[i].firstname + " " + people[i].lastname);
        console.log(people[i].rcsid);
        console.log(people[i].timestamp);
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

        console.log(people[i].firstname + " " + people[i].lastname);
        console.log(people[i].rcsid);
        console.log(people[i].entering);
        console.log(people[i].timestamp);
    }
}