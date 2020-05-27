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
    $("#description").parent().attr("class", "formInput");
    $.post("/new_tracking_id").done(function(result) {
        $("#imageURL").html("<b>Image URL:</b> https://dysmey.herokuapp.com/track/<span id=\"trackingID\">" + result + "</span>");
        $('#addTracker').show("slow");
        $("#description").focus();
    });
});

$("#createNewTracker").click(function() {
    $(this).prop("disabled", true);
    if ($("#description").val().length > 300) {
        $("#description").parent().attr("class", "formInput error");
        $(this).prop("disabled", false);
        return;
    }
    $("#description").parent().attr("class", "formInput");
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

$('.overlay, #addTracker .textButton, #deleteConfirmation .textButton').click(function() {
    $(".dialog").hide("slow", function() {
        $(".overlay").hide()
    });
});

$(".tracker.expandable section").click(function(e) {
    if (["material-icons editTracker", "material-icons deleteTracker"].indexOf(e.target.className) === -1) {
        $(this).siblings("div").toggle();
    }
});

$(".editTracker").click(function() {
    if (!$(this).hasClass("disabled") && $(this).siblings(".description").attr("contenteditable") === "true") {
        showToast("Please wait...", 3000);
        // Update description
        $.post("/update_description", {
            tracking_id: $(this).siblings(".trackerID").text(),
            description: $(this).siblings(".description").text()
        }).done(function(result) {
            console.log(result);
            if (result === "Success") {
                location.reload();
            } else {
                showToast(result, 10000);
            }
        });
    } else if (!$(this).hasClass("disabled")) {
        $(".editTracker, .deleteTracker").addClass("disabled");
        $(this).siblings(".description").attr("contenteditable", true);
        $(this).removeClass("disabled");
        $(this).siblings(".deleteTracker").removeClass("disabled");
        $(this).text("check");
        $(this).siblings(".deleteTracker").text("clear");
    }
});

$(".deleteTracker").click(function() {
    if (!$(this).hasClass("disabled") && $(this).siblings(".description").attr("contenteditable") === "true") {
        // Undo contenteditable
        $(".disabled").removeClass("disabled");
        $(this).siblings(".description").attr("contenteditable", false);
        $(this).siblings(".description").text($(this).siblings(".description").attr("data-description"));
        $(this).text("delete");
        $(this).siblings(".editTracker").text("edit");
    } else if (!$(this).hasClass("disabled")) {
        $(".overlay").show();
        $("#deleteConfirmation").show("slow");
        $("#confirmDelete").attr("data-tracking-id", $(this).siblings(".trackingID").text());
    }
});

$("#confirmDelete").click(function() {
    showToast("Please wait...", 3000);
    $.post("/delete_tracker", {
        tracking_id: $(this).attr("data-tracking-id"),
    }).done(function(result) {
        console.log(result);
        if (result == "Success") {
            location.reload();
        } else {
            showToast(result, 10000);
        }
    });
});

$(".time").each(function() {
    // Convert the timestamps to human-readable text
    var date = new Date(parseInt($(this).text()) * 1000);
    $(this).text(
        date.getFullYear()
        + "-"
        + (date.getMonth() + 1)
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
