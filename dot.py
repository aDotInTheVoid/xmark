#!/usr/bin/env python
import json
import sys
from graphviz import Digraph

with open("./target/doc/xmark.json") as f:
    inp = json.load(f)

dot = Digraph("api", format="svg", engine="dot")
root = inp["index"][inp["root"]]


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

    def add_node(self, idx):
        name = self.inp["index"][idx]["name"]
        path = "::".join(self.inp["paths"][idx]["path"])
        self.dot.node(idx, label=path)

    ###########################################################################
    # Item Visitors                                                           #
    ###########################################################################
    def visit_module(self, module):
        for idx in module["inner"]["items"]:
            self.visit(idx)

    def visit_struct(self, struct):
        self.add_node(struct["id"])
        for idx in struct["inner"]["impls"]:
            self.visit(idx)
    
    def visit_impl(self, impl):
        self.visit_module(impl)

    def visit_method(self, method):
        # And this is a pain, because for Vec<T>, (T, T),
        # &T, &[T], we want T, and that normalizarion is hard.
        idx = method["id"]
        decl = method["inner"]["decl"]
        self.add_node(idx)
        for arg in decl["inputs"]:
     	    if isinstance
        

v = ItemVisitor(inp, dot)
v.visit(inp["root"])

print(root)

v.dot.render()
