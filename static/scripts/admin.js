$('.formInput input').each(function() {
    if ($(this).val() !== '') {
        $('#'+this.id+' + label').animate({
            'fontSize': '0.8rem',
            'top': '1.4rem',
            'padding': '0.25rem'
        }, 80);
    }
    $(this).focusin(() => {
        $('#'+this.id+' + label').animate({
            'fontSize': '0.8rem',
            'top': '1.4rem',
            'padding': '0.25rem'
        }, 80);
    });
    $(this).focusout(function() {
        if ($(this).val() === '') {
            $('#'+this.id+' + label').animate({
                'fontSize': '1rem',
                'top': '2.9rem',
                'padding': 0
            }, 80);
        }
    });
});

$("#signinForm").submit(e => {
    e.preventDefault();
    $("#signinButton").prop("disabled", true);
    showToast("Please wait...", 5000);
    $.post("/admin_signin", { value: $("#password").val() }).done(result => {
        console.log(result);
        if (result == "Success") {
            location.reload();
        } else {
            showToast(result, 10000);
            $("#signinButton").prop("disabled", false);
        }
    });
});

if (typeof trackers !== "undefined") {
    let today = new Date().setHours(0, 0, 0, 0);
    let trackers_today = 0;
    let requests_today = 0;

    for (let i = 0; i < trackers.length; i++) {
        let date = new Date(trackers[i] * 1000).setHours(0, 0, 0, 0);
        if (date === today) {
            trackers_today++;
        }
    }

    $("#trackersToday").text(trackers_today);

    for (let i = 0; i < requests.length; i++) {
        let date = new Date(requests[i] * 1000).setHours(0, 0, 0, 0);
        if (date === today) {
            requests_today++;
        }
    }

    $("#requestsToday").text(requests_today);
}

$("#signoutButton").click(function() {
    showToast("Please wait...");
    $.ajax({
        type: "POST",
        url: "/signout_admin",
        success: function(result) {
            console.log(result);
            if (result == "Success") {
                window.location.href = "/";
            }
        }
    });
});
