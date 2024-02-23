function getCookie(cname) {
    let name = cname + "=";
    let decoded = decodeURIComponent(document.cookie);
    let arr = decoded.split(";");

    for (let i = 0; i < arr.length; i++) {
        let cookie = arr[i];
        while (cookie.charAt(0) == ' ') {
            cookie = cookie.substring(1);
        }
        if (cookie.indexOf(name) == 0) {
            return cookie.substring(name.length, cookie.length);
        }
    }
    return "";
}

function addCookie(cname, cval, days=null) {
    let expires = 0;
    if (days != null) {
        const d = new Date();
        d.setTime(d.getTime() + (days*24*60*60*1000));
        expires = "expires="+ d.toUTCString();
    }
    document.cookie = days == null ?
        cname + "=" + cval + ";path=/;SameSite=Lax" :
        cname + "=" + cval + ";" + expires + ";path=/;SameSite=Lax";
}