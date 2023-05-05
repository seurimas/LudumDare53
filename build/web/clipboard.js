window.set_clipboard_text_js = (text) => {
    console.log('Setting clipboard text to: ' + text);
    document.getElementById('clipboard').value = text;
}

window.get_clipboard_text_js = () => {
    console.log('Getting clipboard text: ' + document.getElementById('clipboard').value);
    return document.getElementById('clipboard').value;
}

window.show_clipboard = (top, left) => {
    document.getElementById('clipboard-container').style.display = 'block';
    document.getElementById('clipboard-container').style.top = top;
    document.getElementById('clipboard-container').style.left = left;
}

window.hide_clipboard = () => {
    document.getElementById('clipboard-container').style.display = 'none';
}

window.save_game_js = (file_name, game) => {
    const a = document.createElement("a");
    const file = new Blob([game], { type: 'application/json' });
    a.href = URL.createObjectURL(file);
    a.download = file_name;
    a.click();
}

window.show_load = () => {
    document.getElementById('load-container').style.display = 'block';
}

window.hide_load = () => {
    document.getElementById('load-container').style.display = 'none';
}

window.set_loader = (loader) => {
    document.getElementById('load').addEventListener('change', (event) => {
        const file = event.target.files[0];
        const reader = new FileReader();
        reader.onload = function (event) {
            loader(reader.result);
        };
        reader.readAsText(file);
    });
}