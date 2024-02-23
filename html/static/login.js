function login_attempt() {
    addCookie("rmusername", document.getElementsByName("username")[0].value);
    addCookie("rmpassword", document.getElementsByName("password")[0].value);
}

function setup() {
    let form = document.getElementById("loginform");
    form.onsubmit = (ev) => {
        login_attempt();
        document.location.reload();
        return false;
    };
}