from flask import (
    render_template,
    request,
    send_file,
    send_from_directory,
    url_for,
    Flask,
    flash,
    redirect,
)
import librisc16_rs

app = Flask(__name__)


@app.route("/", methods=["GET", "POST"])
def index():
    if request.method == "POST":
        max_instr = int(request.form.get("exec"))
        test_file = request.form.get("exo")
        archi = request.form.get("archi")
        unsigned = int(request.form.get("logic"))

        print(request.form)

        file = request.files["file"]
        text = file.read().decode()
        # print(text)
        try:
            code = librisc16_rs.load_rom_py(text)
        except BaseException as e:
            code = e
        res = librisc16_rs.run_from_str_py(text)
        context = {
            "status": "done",
            "log_content": res,
            "code_content": code,
        }
        return render_template("log_done.html", **context)

    return render_template("index.html")

    return dict()
