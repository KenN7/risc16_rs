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
import modules

app = Flask(__name__)


@app.route("/submit", methods=["GET", "POST"])
def upload():
    if request.method == "POST":
        max_instr = int(request.form.get("exec", 100000))
        test_file = request.form.get("exo", "")
        archi = request.form.get("archi", "IS0")  # not used yet
        unsigned = request.form.get("logic", "unsigned")  # not used yet

        print(request.form)

        file = request.files.get("file")
        if file:
            text = file.read().decode()
            print("file")
        else:
            text = request.form.get("code_area", "")

        ex = modules.exercise(test_file)
        input_vec, output_vec = ex.get_input_test_vector()
        tests = librisc16_rs.test_batch_py(max_instr, text, input_vec)

        tests_passed = ex.verify(tests)
        res = ex.create_report(tests_passed, tests)

        try:
            code = librisc16_rs.load_rom_py(text)
        except BaseException as e:
            print(e)
            code = str(e)

        # try:
        #     res, trace = librisc16_rs.run_from_str_py(text)
        # except BaseException as e:
        #     print(e)
        #     res = "Error (see above)"
        #     trace = str(e)

        print(tests)

        context = {
            "tests_results": res,
            "code_content": code,
            # "end_state": trace,
        }
        return context


@app.route("/exercices", methods=["GET"])
@app.route("/", methods=["GET"])
def exos():
    ex = modules.exercise()
    return render_template("index.html", unit=False, list_exo=ex.get_exercices())


@app.route("/unit", methods=["GET"])
def unit():
    return render_template("index.html", unit=True)
