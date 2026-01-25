import { defineStore } from "pinia";

export const useCommandStore = defineStore("command", {
  state: () => ({
    commands: [
      {
        title: "Apply",
        icon: "ri:stack-line",
        description:
          "Apply a series of transformation functions to CSV column/s",
        route: "/command/components/apply"
      },
      {
        title: "Cat",
        icon: "ri:merge-cells-vertical",
        description:
          "Merge multiple CSV or Excel files into one CSV or xlsx file",
        route: "/command/components/cat"
      },
      {
        title: "Count",
        icon: "ri:numbers-line",
        description: "Count the rows of CSV files",
        route: "/command/components/count"
      },
      {
        title: "Convert",
        icon: "ri:exchange-2-line",
        description: "File type conversion",
        route: "/command/components/convert"
      },
      {
        title: "Rename",
        icon: "ri:heading",
        description: "Rename the columns of a CSV",
        route: "/command/components/rename"
      },
      {
        title: "Select",
        icon: "ri:check-double-line",
        description: "Select, re-order columns",
        route: "/command/components/select"
      },
      {
        title: "Separate",
        icon: "ri:menu-search-line",
        description: "Separate CSV into good and bad rows",
        route: "/command/components/separate"
      },
      {
        title: "Search",
        icon: "ri:filter-2-line",
        description: "Select fields matching rows",
        route: "/command/components/search"
      },
      {
        title: "Fill",
        icon: "ri:rhythm-fill",
        description: "Fill empty fields in selected columns of a CSV",
        route: "/command/components/fill"
      },
      {
        title: "Split",
        icon: "ri:split-cells-horizontal",
        description: "Split one CSV file into many CSV files",
        route: "/command/components/split"
      },
      {
        title: "Skip",
        icon: "ri:skip-up-line",
        description: "Skip rows from CSV",
        route: "/command/components/skip"
      },
      {
        title: "Slice",
        icon: "ri:timeline-view",
        description: "Returns rows of a CSV file in the specified range",
        route: "/command/components/slice"
      },
      {
        title: "Enumerate",
        icon: "ri:sort-number-asc",
        description: "Add a new column enumerating the lines of a CSV file",
        route: "/command/components/enumerate"
      },
      {
        title: "Pinyin",
        icon: "ri:pinyin-input",
        description: "Convert Chinese to Pinyin for specific column in CSV",
        route: "/command/components/pinyin"
      },
      {
        title: "Replace",
        icon: "ri:find-replace-line",
        description: "Replace occurrences of a pattern across a CSV file",
        route: "/command/components/replace"
      },
      {
        title: "Join",
        icon: "ri:merge-cells-horizontal",
        description: "Joins two sets of CSV data on the specified columns",
        route: "/command/components/join"
      },
      {
        title: "Sort",
        icon: "ri:sort-alphabet-asc",
        description: "Sorts CSV data lexicographically",
        route: "/command/components/sort"
      },
      {
        title: "String",
        icon: "ri:formula",
        description: "String expr: slice, split, pad...",
        route: "/command/components/string"
      },
      {
        title: "Reverse",
        icon: "ri:arrow-up-down-line",
        description: "Reverses rows of CSV data",
        route: "/command/components/reverse"
      },
      {
        title: "Transpose",
        icon: "ri:loop-left-line",
        description: "Transpose rows/columns of a CSV",
        route: "/command/components/transpose"
      },
      {
        title: "Traverse",
        icon: "ri:align-right",
        description: "Traverse the directory to obtain filenames",
        route: "/command/components/traverse"
      }
    ]
  })
});
