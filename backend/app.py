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

        file = request.files.get("file")
        if file:
            text = file.read().decode()
        else:
            text = request.form.get("code_area")

        # print(text)
        try:
            code = librisc16_rs.load_rom_py(text)
        except BaseException as e:
            print(e)
            code = str(e)

        try:
            res, trace = librisc16_rs.run_from_str_py(text)
        except BaseException as e:
            print(e)
            res = "Error (see above)"
            trace = str(e)

        context = {
            "tests_results": res,
            "code_content": code,
            "end_state": trace,
        }
        return context


@app.route("/exercices", methods=["GET"])
@app.route("/", methods=["GET"])
def exos():
    return render_template("index.html")


@app.route("/unit", methods=["GET"])
def unit():
    return render_template("index_unit.html")
