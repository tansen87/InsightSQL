import { nextTick, onMounted, onUnmounted, Ref, ref, watch } from "vue";
import { useDark } from "@pureadmin/utils";
import { marked } from "marked";
import Prism from "prismjs";
import "prismjs/components/prism-sql";

const themeMap = {
  dark: "prism-tomorrow.css",
  light: "prism-solarizedlight.css"
};

const loadedThemes = new Set<string>();

function usePrismTheme(isDark: Ref<Boolean>) {
  const loadTheme = async (themeType: "dark" | "light") => {
    const themePath = themeMap[themeType];

    if (loadedThemes.has(themePath)) return;

    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = `/node_modules/prismjs/themes/${themePath}`;

    link.onload = () => {
      loadedThemes.add(themePath);
      Prism.highlightAll();
    };

    document.head.appendChild(link);
  };

  const removeOldTheme = (themeType: "dark" | "light") => {
    const oldTheme = themeMap[themeType];
    document.head.querySelectorAll("link").forEach(link => {
      if (link.href.endsWith(oldTheme)) {
        link.remove();
        loadedThemes.delete(oldTheme);
      }
    });
  };

  watch(
    isDark,
    newVal => {
      const targetTheme = newVal ? "dark" : "light";
      removeOldTheme(newVal ? "light" : "dark");
      loadTheme(targetTheme);
    },
    { immediate: true }
  );

  onUnmounted(() => {
    Object.values(themeMap).forEach(theme => {
      removeOldTheme(theme === themeMap.dark ? "dark" : "light");
    });
  });
}

export function useMarkdown(initialMarkdownFn: () => string) {
  const { isDark } = useDark();
  const markdownContent = ref(initialMarkdownFn());
  const compiledMarkdown = ref(marked.parse(markdownContent.value));

  usePrismTheme(isDark);

  const highlightCode = async () => {
    await nextTick();
    Prism.highlightAll();
  };

  onMounted(() => {
    highlightCode();
  });

  watch(markdownContent, async newContent => {
    compiledMarkdown.value = marked.parse(newContent);
    await highlightCode();
  });

  return {
    markdownContent,
    compiledMarkdown,
    updateMarkdownContent: (newMarkdownFn: () => string) => {
      markdownContent.value = newMarkdownFn();
    }
  };
}

export function applyContent() {
  return `
  ### OPERATIONS
  | Operations | Description                                       |
  | :--------- | :------------------------------------------------ |
  | Copy       | mark a column for copying                         |
  | Len        | return string length                              |
  | Lower      | transform to lowercase                            |
  | Upper      | transform to uppercase                            |
  | Trim       | trim (drop whitespace left & right of the string) |
  | Ltrim      | left trim whitespace                              |
  | Rtrim      | right trim whitespace                             |
  | Replace    | replace all matches of a pattern                  |
  | Round      | round off, AKA "Bankers Rounding"                 |
  | Squeeze    | compress consecutive whitespaces                  |
  | Strip      | replace \\ r and \\ n with whitespaces            |
  
  ### DYNFMT
  Dynamically constructs a new column from other columns using the <--formatstr> template.

  ### CALCCONV
  Parse and evaluate math expressions into a new column, with support for units and conversions.
  The math expression is built dynamically using the <--formatstr> template, similar to the DYNFMT.
`;
}

export function catContent() {
  return `
\`\`\`
sample file
test1.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘

test2.xlsx (test2.csv)
┌─────┬─────┐
│ age │ idx │
├─────┼─────┤
│ 10  │  4  │
│ 21  │  5  │
│ 31  |  6  |
└─────┴─────┘
\`\`\`

### 1. Polars (support csv and excel file)
\`\`\`
┌─────┬────────┬─────────────┬─────┐
│ idx │ name   │ _filename_  │ age │
├─────┼────────┼─────────────┼─────┤
│  1  │ tom    │ test1.csv   │     │
│  2  │ jerry  │ test1.csv   │     │
│  3  | hansen | test1.csv   │     │
│  4  │        │ test2.xlsx  │ 10  │
│  5  │        │ test2.xlsx  │ 21  │
│  6  │        │ test2.xlsx  │ 31  │
└─────┴────────┴─────────────┴─────┘
\`\`\`

### 2. CSV (only support csv file)
\`\`\`
┌─────┬────────┬─────┐
│ idx │ name   │ age │
├─────┼────────┤─────┤
│  1  │ tom    │     │
│  2  │ jerry  │     │
│  3  | hansen |     │
│  4  │        │ 10  │
│  5  │        │ 21  │
│  6  │        │ 31  │
└─────┴────────┴─────┘
\`\`\`

### 3. Duplicate (only support csv file)
\`\`\`
sample file (test.csv)
┌──────┬───────┐
│ name │ name  │
├──────┼───────┤
│  1   │ tom   │
└──────┴───────┘

duplicate result: {"name"}
\`\`\`
`;
}

export function splitContent() {
  return `
  Split **file.csv**, set the **Split rows** to \`2\`,
  then we get two files, As show in the figure:
  ![split.png](/demo/split.png)
`;
}

export function sliceContent() {
  return `
\`\`\`
sample file
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│  1  │ tom-1    │
│  2  │ jerry-2  │
│  3  | hansen-3 |
└─────┴──────────┘
\`\`\`

### 1. Left
Set criteria (Select column: <u>name</u>, Number of the string: <u>3</u>, String mode: <u>Left</u>)
\`\`\`
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    tom     │
│  2  │ jerry-2  │    jer     │
│  3  | hansen-3 |    han     │
└─────┴──────────┴────────────┘
\`\`\`

### 2. Right
Set criteria (Select column: <u>name</u>, Number of the string: <u>3</u>, String mode: <u>Right</u>)
\`\`\`
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    m-1     │
│  2  │ jerry-2  │    y-2     │
│  3  | hansen-3 |    n-3     │
└─────┴──────────┴────────────┘
\`\`\`

### 3. StartLength
Set criteria (Select column: <u>name</u>, Start index: <u>1</u>, Length of the string: <u>3</u>, String mode: <u>StartLength</u>)
\`\`\`
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   tom   │
│  2  │ jerry-2  │   jer   │
│  3  | hansen-3 |   han   │
└─────┴──────────┴─────────┘
\`\`\`
Set criteria (Select column: <u>name</u>, Start index: <u>-1</u>, Length of the string: <u>3</u>, String mode: <u>StartLength</u>)
\`\`\`
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   m-1   │
│  2  │ jerry-2  │   y-2   │
│  3  | hansen-3 |   n-3   │
└─────┴──────────┴─────────┘
\`\`\`

### 4. Nth
Set criteria (Select column: <u>name</u>, Number of the string: <u>1</u>, String separator: <u>-</u>, String mode: <u>Nth</u>)
\`\`\`
┌─────┬──────────┬──────────┐
│ idx │ name     │ name_nth │
├─────┼──────────┼──────────┤
│  1  │ tom-1    │  tom     │
│  2  │ jerry-2  │  jerry   │
│  3  | hansen-3 |  hansen  │
└─────┴──────────┴──────────┘
\`\`\`

### 5. Nmax
Set criteria (Select column: <u>name</u>, Number of the string: <u>2</u>, String separator: <u>-</u>, String mode: <u>Nmax</u>)
\`\`\`
┌─────┬──────────┬────────────┬────────────┐
│ idx │ name     │ name_nmax1 │ name_nmax2 │
├─────┼──────────┼────────────┼────────────┤
│  1  │ tom-1    │  tom       │     1      │
│  2  │ jerry-2  │  jerry     │     2      │
│  3  | hansen-3 |  hansen    │     3      │
└─────┴──────────┴────────────┴────────────┘
\`\`\`
`;
}

export function searchContent() {
  return `
\`\`\`
sample file (test.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 1. Equal
Set criteria (Select column: <u>name</u>, Search mode: <u>Equal</u>, Search conditions: <u>tom|jerry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 2. EqualMulti
Set criteria (Select column: <u>name</u>, Search mode: <u>EqualMulti</u>, Search conditions: <u>tom|jerry</u>)
\`\`\`
test_tom.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jerry.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 3. NotEqual
Set criteria (Select column: <u>name</u>, Search mode: <u>NotEqual</u>, Search conditions: <u>tom|jerry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 4. Contains
Set criteria (Select column: <u>name</u>, Search mode: <u>Contains</u>, Search conditions: <u>om|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 5. ContainsMulti
Set criteria (Select column: <u>name</u>, Search mode: <u>ContainsMulti</u>, Search conditions: <u>om|jer</u>)
\`\`\`
test_om.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jer.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 6. NotContains
Set criteria (Select column: <u>name</u>, Search mode: <u>NotContains</u>, Search conditions: <u>om|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 7. StartsWith
Set criteria (Select column: <u>name</u>, Search mode: <u>StartsWith</u>, Search conditions: <u>to|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 8. StartsWithMulti
Set criteria (Select column: <u>name</u>, Search mode: <u>StartsWithMulti</u>, Search conditions: <u>to|jer</u>)
\`\`\`
test_to.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jer.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 9. NotStartsWith
Set criteria (Select column: <u>name</u>, Search mode: <u>NotStartsWith</u>, Search conditions: <u>to|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 10. EndsWith
Set criteria (Select column: <u>name</u>, Search mode: <u>EndsWith</u>, Search conditions: <u>om|rry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 11. EndsWithMulti
Set criteria (Select column: <u>name</u>, Search mode: <u>EndsWithMulti</u>, Search conditions: <u>om|rry</u>)
\`\`\`
test_om.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_rry.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 12. NotEndsWith
Set criteria (Select column: <u>name</u>, Search mode: <u>NotEndsWith</u>, Search conditions: <u>om|rry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 13. Regex
Set criteria (Select column: <u>name</u>, Search mode: <u>Regex</u>, Search conditions: <u>hansen</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 14. IsNull
Set criteria (Select column: <u>name</u>, Search mode: <u>IsNull</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│     │        │
└─────┴────────┘
\`\`\`

### 15. IsNotNull
Set criteria (Select column: <u>name</u>, Search mode: <u>IsNotNull</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 16. gt
Set criteria (Select column: <u>idx</u>, Search mode: <u>gt</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 17. ge
Set criteria (Select column: <u>idx</u>, Search mode: <u>ge</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 18. lt
Set criteria (Select column: <u>idx</u>, Search mode: <u>lt</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘
\`\`\`

### 19. le
Set criteria (Select column: <u>idx</u>, Search mode: <u>le</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`
`;
}
