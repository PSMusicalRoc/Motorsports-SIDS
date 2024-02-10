console.log(window.location.origin);
const websocket = new WebSocket(window.location.origin.replace("http", "ws") + "/websocket");

websocket.onopen = function() {
    this.send("get_all_people");
};

websocket.onmessage = function(ev) {
    try {
        let jsObj = JSON.parse(ev.data);
        
        if (jsObj.msgtype == "null") {
            console.log("Null message");
        } else if (jsObj.msgtype == "message") {
            console.log("Server message: " + jsObj.message);
        } else if (jsObj.msgtype == "in_shop_add") {
            console.log(jsObj);
            let people = JSON.parse(jsObj.message);
            createInShopTable(people);
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

        console.log(people[i].firstname + " " + people[i].lastname);
        console.log(people[i].rcsid);
        console.log(people[i].timestamp);
    }
}