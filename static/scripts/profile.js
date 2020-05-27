$('.formInput input, .formInput select').each(function() {
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
                'top': '3rem',
                'padding': 0
            }, 80);
        }
    });
});

$("#signinForm").submit(function(e) {
    e.preventDefault();
    $("#signinButton").prop("disabled", true);
    showToast("Please wait...", 5000);
    $.ajax({
        type: "POST",
        url: "/signin",
        data: {
            email: $("#email").val(),
            password: $("#password").val(),
        },
        success: function(result) {
            console.log(result);
            if (result == "Success") {
                window.location.href = "/";
            } else {
                showToast(result, 10000);
                $("#signinButton").prop("disabled", false);
            }
        }
    });
});

$("#signupForm").submit(function(e) {
    e.preventDefault();
    $('#signupButton').prop('disabled', true);
    if ($("#password").val().length < 8) {
        $("#confirmPassword").parent().attr("class", "formInput");
        $("#password").parent().attr("class", "formInput error");
        $('#signupButton').prop('disabled', false);
        return;
    }
    if ($("#password").val() !== $("#confirmPassword").val()) {
        $("#password").parent().attr("class", "formInput");
        $("#confirmPassword").parent().attr("class", "formInput error");
        $("#signupButton").prop("disabled", false);
        return;
    }
    showToast("Please wait...", 5000);
    $("#password").parent().attr("class", "formInput");
    $("#confirmPassword").parent().attr("class", "formInput");
    $.ajax({
        type: "POST",
        url: "/signup",
        data: {
            email: $("#email").val(),
            password: $("#password").val(),
        },
        success: function(result) {
            console.log(result);
            if (result == "Success") {
                window.location.href = "/";
            } else {
                showToast(result, 10000);
                $("#signupButton").prop("disabled", false);
            }
        }
    });
});

$("#profilePicture").click(function() {
    $(".overlay").show();
    $("#profilePictureInfo").show("slow");
});

$("#addTrackerButton").click(function() {
    $(".overlay").show();
    $("#description").val("");
    $.post("/new_tracking_id").done(function(result) {
        $("#imageURL").html("<b>Image URL:</b> https://dysmey.herokuapp.com/track/<span id=\"trackingID\">" + result + "</span>");
        $('#addTracker').show("slow");
        $("#description").focus();
    });
});

$("#createNewTracker").click(function() {
    $(this).prop("disabled", true);
    $.ajax({
        type: "POST",
        url: "/register_tracker",
        data: {
            tracking_id: $("#trackingID").text(),
            description: $("#description").val(),
        },
        success: function(result) {
            console.log(result);
            if (result == "Success") {
                location.reload();
            } else {
                showToast(result, 10000);
                $("#createNewTracker").prop("disabled", false);
            }
        }
    });
});

$('.overlay, #addTracker .textButton').click(function() {
    $(".dialog").hide("slow", function() {
        $(".overlay").hide()
    });
});

$(".tracker.expandable section").click(function() {
    $(this).siblings("div").toggle();
});

$(".time").each(function() {
    // Convert the timestamps to human-readable text
    var date = new Date(parseInt($(this).text()) * 1000);
    $(this).text(
        date.getFullYear()
        + "-"
        + date.getMonth()
        + 1
        + "-"
        + date.getDate()
        + " "
        + date.getHours().toString().padStart(2, 0)
        + ":"
        + date.getMinutes().toString().padStart(2, 0)
    );
});

$("#signoutButton").click(function() {
    showToast("Please wait...");
    $.ajax({
        type: "POST",
        url: "/signout",
        success: function(result) {
            console.log(result);
            if (result == "Success") {
                window.location.href = "/";
            }
        }
    });
});
