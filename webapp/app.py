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
        trace_bool = request.form.get("trace", 0) == "1"
        trace = ""
        # print(request.form)

        file = request.files.get("file")
        if file and test_file != "":
            text = file.read().decode()
            ex = modules.exercise(test_file)
            input_vec, output_vec = ex.get_input_test_vector()
            try:
                # tests = librisc16_rs.test_batch_py(
                #     max_instr, trace_bool, text, input_vec
                # )
                tests = librisc16_rs.test_batch_par_py(
                    max_instr, trace_bool, text, input_vec
                )
                tests_passed = ex.verify([reg.registers for reg in tests])
                res = ex.create_report(tests_passed, tests)
            except BaseException as e:
                # print(e)
                res = str(e)
                trace = "Error!"
        else:
            text = request.form.get("code_area", "")
            try:
                res, trace = librisc16_rs.run_from_str_py(max_instr, trace_bool, text)
                # print(res, trace)
            except BaseException as e:
                # print(e)
                res = str(e)
                trace = "Error!"

        try:
            code = librisc16_rs.load_rom_py(text)
        except BaseException as e:
            # print(e)
            code = str(e)

        context = {
            "tests_results": res,
            "code_content": code,
            "end_state": trace,
        }
        return context


@app.route("/exercices", methods=["GET"])
@app.route("/", methods=["GET"])
def exos():
    return render_template("index.html", unit=False, list_exo=modules.get_exercices())


@app.route("/unit", methods=["GET"])
def unit():
    return render_template("index.html", unit=True)
