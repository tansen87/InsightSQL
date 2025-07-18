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
  Merge **file1.csv** and **file2.csv**, then we get **cat.csv**.
  As shown in the figure:
  ![cat.png](/demo/cat.png)
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
