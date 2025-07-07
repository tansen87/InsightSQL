import { defineStore } from "pinia";

export const useCommandStore = defineStore("command", {
  state: () => ({
    commands: [
      {
        title: "Apply",
        description:
          "Apply a series of transformation functions to CSV column/s.",
        route: "/command/components/apply"
      },
      {
        title: "Cat",
        description:
          "Merge multiple CSV or Excel files into one CSV or xlsx file.",
        route: "/command/components/cat"
      },
      {
        title: "2csv",
        description: "Batch convert files to CSV",
        route: "/command/components/toCsv"
      },
      {
        title: "Count",
        description: "Count the rows of CSV files.",
        route: "/command/components/count"
      },
      {
        title: "2excel",
        description: "Batch convert files to Excel",
        route: "/command/components/toExcel"
      },
      {
        title: "Rename",
        description: "Rename the columns of a CSV.",
        route: "/command/components/rename"
      },
      {
        title: "Select",
        description: "Select, re-order columns.",
        route: "/command/components/select"
      },
      {
        title: "Search",
        description: "Select fields matching rows.",
        route: "/command/components/search"
      },
      {
        title: "Fill",
        description: "Fill empty fields in selected columns of a CSV.",
        route: "/command/components/fill"
      },
      {
        title: "Split",
        description: "Split one CSV file into many CSV files.",
        route: "/command/components/split"
      },
      {
        title: "Access to CSV",
        description: "Convert Access Database to CSV.",
        route: "/command/components/access"
      },
      {
        title: "Dbf to CSV",
        description: "Convert dbf file to CSV.",
        route: "/command/components/dbf"
      },
      {
        title: "Skip",
        description: "Skip rows from CSV.",
        route: "/command/components/skip"
      },
      {
        title: "Enumerate",
        description: "Add a new column enumerating the lines of a CSV file.",
        route: "/command/components/enumerate"
      },
      {
        title: "Chinese to Pinyin",
        description: "Convert Chinese to Pinyin for specific column in CSV.",
        route: "/command/components/pinyin"
      },
      {
        title: "Replace",
        description: "Replace occurrences of a pattern across a CSV file.",
        route: "/command/components/replace"
      },
      {
        title: "Join",
        description: "Joins two sets of CSV data on the specified columns.",
        route: "/command/components/join"
      },
      {
        title: "Sort",
        description: "Sorts CSV data lexicographically.",
        route: "/command/components/sort"
      },
      {
        title: "Slice",
        description: "Slicing of CSV column",
        route: "/command/components/slice"
      },
      {
        title: "Reverse",
        description: "Reverses rows of CSV data.",
        route: "/command/components/reverse"
      },
      {
        title: "Transpose",
        description: "Transpose rows/columns of a CSV.",
        route: "/command/components/transpose"
      },
      {
        title: "Traverse",
        description: "Traverse the directory to obtain filenames.",
        route: "/operation/components/traverse"
      }
    ]
  })
});
