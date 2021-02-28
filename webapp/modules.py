import re
import os

MODULE_FOLDER = "modules/"


def get_exercices():
    return os.listdir(MODULE_FOLDER)


class exercise:
    def __init__(self, exercise):
        self.pattern = re.compile(r"r(?P<registerin>\d)=(?P<value>\w+);")
        self.failpattern = ""
        self.passpattern = ""
        self.inputs = []
        self.outputs = []
        self.exercise = exercise
        self.create_input_test_vector()

    def create_input_test_vector(self):
        in_filename = os.path.join(MODULE_FOLDER, self.exercise)
        in_file = open(in_filename, "r")
        for line in in_file:
            if line.startswith("in:"):
                li = self.pattern.findall(line)
                if li != []:
                    self.inputs.append([(int16i(i), int16i(j)) for i, j in li])
            elif line.startswith("out:"):
                li = self.pattern.findall(line)
                if li != []:
                    self.outputs.append([(int16i(i), int16i(j)) for i, j in li])
            elif line.startswith("# fail:"):
                self.failpattern = line[len("# fail:") :]
            elif line.startswith("# pass:"):
                self.passpattern = line[len("# pass:") :]
        in_file.close()

    def get_input_test_vector(self):
        return self.inputs, self.outputs

    def verify(self, tests):
        # print(outputs, inputs, tests)
        results = []
        for i, test_runs in enumerate(self.outputs):
            passed = True
            for output in test_runs:
                if tests[i][output[0]] != output[1]:
                    passed = False
                    # print("failed", output, tests[i])
            results.append((i, passed))
        return results

    def create_report(self, results, tests):
        l = []
        for r in results:
            d = {}
            for reg in self.inputs[r[0]]:
                d[f"ri{reg[0]}"] = to_16b_signed_hex(reg[1])
            for reg in self.outputs[r[0]]:
                d[f"ro{reg[0]}"] = to_16b_signed_hex(reg[1])
            for i, reg in enumerate(tests[r[0]].registers):
                d[f"risc{i}"] = to_16b_signed_hex(reg)
            if r[1]:  # passed
                l.append(
                    risc16_to_json(tests[r[0]], True, self.passpattern.format(**d))
                )
            else:  # failed
                l.append(
                    risc16_to_json(tests[r[0]], False, self.failpattern.format(**d))
                )
        return l


# function used to serialize risc 16 proc objects to dict so that
# flask can json serialize it.
def risc16_to_json(proc, test_bool, test_str):
    return {
        "buffer": proc.buffer,
        "instr_count": proc.instr_count,
        "labels": proc.labels,
        "pc": proc.pc,
        "registers": proc.registers,
        "test": test_bool,
        "result_str": test_str,
    }


# dirty hack for converting hax unsigned value to their i16 counterpart
# This is because python always assumes hex coverted values are positive
def int16i(i):
    b = 2 ** 16 // 2
    try:
        n = int(i, 0)
    except TypeError:
        n = int(i)
    if n >= b:
        return n % b - b
    return n


# convert back negative numbers to their 16 bits 2's complement notation
def to_16b_signed_hex(i):
    i = int16i(i)
    if i < 0:
        b = 2 ** 16
        return i + b
    return i


if __name__ == "__main__":
    pass
