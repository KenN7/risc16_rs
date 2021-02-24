// Small js script to handle uploading and syntax highlighting

function process_risc(event) {
    let code_pre = document.getElementById("code_result");
    let log = document.getElementById("tests_results");
    let end_state = document.getElementById("end_state");

    let d = JSON.parse(event.target.response)
    console.log(d)
    if (code_pre) {
        code_pre.innerHTML = d.code_content
    }
    if (log) {
        log.innerHTML = d.tests_results
    }
    if (end_state) {
        end_state.innerHTML = d.end_state
    }

}

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


window.addEventListener("load", function () {
    function sendData() {
        let XHR = new XMLHttpRequest()
        let FD = new FormData(form)
        // Définissez ce qui se passe si la soumission s'est opérée avec succès
        XHR.addEventListener("load", function (event) {
            process_risc(event)
            highlight()
        });
        // Definissez ce qui se passe en cas d'erreur
        XHR.addEventListener("error", function (event) {
            alert('Something went wrong with your request.')
        })
        // Configurez et envoyez la requête
        XHR.open("POST", "http://127.0.0.1:5000/submit")
        XHR.send(FD)
    }

    // Accédez à l'élément form
    let form = document.getElementById("submit_code")
    form.addEventListener("submit", function (event) {
        event.preventDefault()
        sendData()
    });

    // highlight the code:
    highlight()
})

