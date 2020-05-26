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
    $('#signinButton').prop('disabled', true);
    showToast('Please wait...', 5000);
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
                $('#signinButton').prop('disabled', false);
            }
        }
    });
});

$("#signupForm").submit(function(e) {
    e.preventDefault();
    $('#signupButton').prop('disabled', true);
    showToast('Please wait...', 5000);
    if ($("#password").val().length < 8) {
        $("#confirmPassword").parent().attr("class", "formInput");
        $("#password").parent().attr("class", "formInput error");
        $('#signupButton').prop('disabled', false);
        return;
    }
    if ($("password").val() !== $("#confirmPassword").val()) {
        $("#password").parent().attr("class", "formInput");
        $("#confirmPassword").parent().attr("class", "formInput error");
        $('#signupButton').prop('disabled', false);
        return;
    }
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
                $('#signupButton').prop('disabled', false);
            }
        }
    });
});
