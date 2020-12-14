#!/usr/bin/env python
import json
import sys
from graphviz import Digraph


class Digraph2(Digraph):
    def node2(self, name, label=None, _attributes=None, **attrs):
        self.node(self._quote2(name), label, _attributes, **attrs)

    def edge2(self, tail_name, head_name, label=None, _attributes=None, **attrs):
        self.edge(
            self._quote2(tail_name),
            self._quote2(head_name),
            label=None,
            _attributes=None,
            **attrs,
        )

    def _quote2(self, var):
        return var.replace(":", "___")


class ItemVisitor:
    def __init__(self, inp, dot):
        self.dot = dot
        self.inp = inp

    def visit(self, idx):
        to_visit = self.inp["index"][idx]
        method_name = "visit_" + to_visit["kind"]
        function = getattr(self, method_name, self.not_found)
        return function(to_visit)

    def not_found(self, item):
        #       return
        raise Exception(f"No method `visit_{item['kind']}`")

    def add_node(self, idx, shape="ellipse"):
        name = self.inp["index"][idx]["name"]
        path = self.inp["paths"].get(idx)

        path = "::".join(path["path"]) if path is not None else name
        self.dot.node2(idx, label=path, shape=shape)

    def extract_ty_id(self, ty):
        kind = ty["kind"]
        inner = ty["inner"]
        if kind == "resolved_path":
            idx = inner["id"]
            if idx.split(":")[0] == "0":
                return idx

        elif (
            kind == "borrowed_ref"
            or kind == "array"
            or kind == "raw_pointer"
            or kind == "borrowed_ref"
        ):
            return self.extract_ty_id(inner["type"])
        elif kind == "qualified_path":
            return self.extract_ty_id(inner["self_type"])

    ###########################################################################
    # Core visitors                                                           #
    ###########################################################################
    def visit_module(self, module):
        for idx in module["inner"]["items"]:
            self.visit(idx)

    def visit_struct(self, struct):
        self.add_node(struct["id"])
        for idx in struct["inner"]["impls"]:
            self.visit(idx)

    def visit_method(self, method):
        # And this is a pain, because for Vec<T>, (T, T),
        # &T, &[T], we want T, and that normalizarion is hard.
        fn_index = method["id"]

        fn_decl = method["inner"]["decl"]

        self.add_node(fn_index, shape="box")

        for [_, in_ty] in fn_decl["inputs"]:
            if in_id := self.extract_ty_id(in_ty):
                self.dot.edge2(in_id, fn_index)

        if out_ty := fn_decl["output"]:
            if out_id := self.extract_ty_id(out_ty):
                self.dot.edge2(fn_index, out_id)

    ###########################################################################
    # Passthoughts and ignores                                                #
    ###########################################################################
    def visit_enum(self, enum):
        self.visit_struct(enum)

    def visit_impl(self, impl):
        self.visit_module(impl)

    def visit_function(self, function):
        self.visit_method(function)

    def visit_import(self, _):
        pass

    def visit_typedef(self, _):
        pass


with open("./target/doc/xmark.json") as f:
    inp = json.load(f)

dot = Digraph2("api", format="svg", engine="dot")
root = inp["index"][inp["root"]]


v = ItemVisitor(inp, dot)
v.visit(inp["root"])


v.dot.render()
