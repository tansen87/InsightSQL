import ace from "ace-builds";
import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/src-noconflict/mode-sql";
import "ace-builds/src-noconflict/theme-chrome";
import "ace-builds/src-noconflict/theme-monokai";

ace.config.set("basePath", "/node_modules/ace-builds/src-noconflict");
ace.require("ace/ext/language_tools");
ace.require("ace-builds/mode-sql");
ace.require("ace-builds/theme-chrome");
ace.require("ace-builds/theme-monokai");
