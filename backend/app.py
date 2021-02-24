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


@app.route("/submit", methods=["GET", "POST"])
def upload():
    if request.method == "POST":
        max_instr = request.form.get("exec")
        test_file = request.form.get("exo")
        archi = request.form.get("archi")
        unsigned = request.form.get("logic")

        print(request.form)

        file = request.files["file"]
        text = file.read().decode()
        # print(text)
        try:
            code = librisc16_rs.load_rom_py(text)
        except BaseException as e:
            code = str(e)

        res = librisc16_rs.run_from_str_py(text)
        context = {
            "tests_results": res,
            "code_content": code,
        }
        return context


@app.route("/", methods=["GET"])
def index():
    return render_template("index.html")
