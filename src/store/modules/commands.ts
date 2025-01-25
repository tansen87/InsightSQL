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
        title: "Excel to csv",
        description: "Batch convert Excel to CSV.",
        route: "/command/components/excel"
      },
      {
        title: "Count",
        description: "Count the rows of CSV files.",
        route: "/command/components/count"
      },
      {
        title: "Csv to xlsx",
        description: "Batch convert CSV to Excel.",
        route: "/command/components/csv"
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
        title: "Drop headers",
        description: "Drop headers from CSV.",
        route: "/command/components/behead"
      },
      {
        title: "Offset",
        description: "net amount.",
        route: "/command/components/offset"
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
        title: "Modify",
        description: "Batch modify filenames.",
        route: "/operation/components/modify"
      },
      {
        title: "Traverse",
        description: "Traverse the directory to obtain filenames.",
        route: "/operation/components/traverse"
      }
    ]
  })
});
