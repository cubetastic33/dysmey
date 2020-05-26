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
