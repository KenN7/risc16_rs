// Small js script to handle uploading and syntax highlighting

// Process results coming from backend
function processRisc(event) {
    let d = JSON.parse(event.target.response)
    console.log(d)

    let code_pre = document.getElementById("code_result");
    if (code_pre) {
        code_pre.textContent = d.code_content
    }

    let end_state = document.getElementById("end_state");
    if (end_state) {
        end_state.textContent = d.end_state
    }

    let tests = document.getElementById("tests_results");
    if (tests && Array.isArray(d.tests_results)) {
        tests.innerHTML = `<p>${d.tests_results.map(t =>
            t.test
                ? `👍 ${t.result_str} (${t.instr_count} instruction(s))`
                : `❌ ${t.result_str} (${t.instr_count} instruction(s))`).join('</p><p>')}</p>`
    } else if (tests) {
        tests.textContent = d.tests_results
    }

}

// Process syntax highlighting
function highlight() {
    const patterns = {
        patterns: [
            {
                name: 'comment',
                match: /^(\/\/.*)/
            },
            {
                name: 'instr',
                match: /^(nop|halt|reset|addi|add|nand|movi|lui|lw|sw|beq|jalr)/i
            },
            {
                name: 'label',
                match: /^([A-z]+:)/
            }
        ]
    }
    window.csHighlight(patterns)
}

// switch mode from dark to light
function switchMode(el) {
    const bodyClass = document.body.classList;
    bodyClass.contains("dark")
        ? ((el.innerHTML = "☀️"), bodyClass.remove("dark"))
        : ((el.innerHTML = "🌙"), bodyClass.add("dark"))
}

function toggleSubmitButton(state) {
    let button = document.getElementById("submit_button")
    button.disabled = state;
    button.classList.contains("button--loading")
        ? button.classList.remove("button--loading")
        : button.classList.add("button--loading")
}

function sendData(form) {
    toggleSubmitButton(true)

    let XHR = new XMLHttpRequest()
    let FD = new FormData(form)
    // Définissez ce qui se passe si la soumission s'est opérée avec succès
    XHR.addEventListener("load", function (event) {
        processRisc(event)
        highlight()
        toggleSubmitButton(false)
    });
    // Definissez ce qui se passe en cas d'erreur
    XHR.addEventListener("error", function (event) {
        alert('Something went wrong with your request.')
        toggleSubmitButton(false)
    })
    // Configurez et envoyez la requête
    XHR.open("POST", "http://127.0.0.1:5000/submit")
    XHR.send(FD)
}

// main function loaded with DOM
window.addEventListener("load", function () {
    // Accédez à l'élément form
    let form = document.getElementById("submit_code")
    form.addEventListener("submit", function (event) {
        event.preventDefault()
        sendData(form)
    });

    // highlight the code:
    highlight()

    //if media prefers dark mode:
    if (
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches
    ) {
        document.body.classList.add("dark");
        document.querySelector('#theme-switch').innerHTML = "🌙"
    }
})

