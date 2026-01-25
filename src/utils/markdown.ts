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

export function useMarkdown(mdInitFn: () => string) {
  const { isDark } = useDark();
  const markdown = ref(mdInitFn());
  const mdShow = ref(marked.parse(markdown.value));

  usePrismTheme(isDark);

  const highlightCode = async () => {
    await nextTick();
    Prism.highlightAll();
  };

  onMounted(() => {
    highlightCode();
  });

  watch(markdown, async newContent => {
    mdShow.value = marked.parse(newContent);
    await highlightCode();
  });

  return {
    markdown,
    mdShow,
    mdUpdate: (newMarkdownFn: () => string) => {
      markdown.value = newMarkdownFn();
    }
  };
}

export function mdApply() {
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
  
  ### DynFmt
  Dynamically constructs a new column from other columns.

  ### CalcConv
  Parse and evaluate math expressions into a new column, with support for units and conversions.
`;
}

export function mdCat() {
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

export function mdSplit() {
  return `
### 1. Rows (standard csv file)
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`
(Split rows: <u>2</u>, Split mode: <u>Rows</u>)
\`\`\`
split 1
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘

split 2
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 2. Lines
\`\`\`
sample file
------------------------
idx,name
hello world...
say hello.
this is a test for lines.
------------------------
\`\`\`
(Split rows: <u>2</u>, Split mode: <u>Lines</u>)
\`\`\`
split 1
------------------------
idx,name
hello world...
say hello.
------------------------

split 2
------------------------
idx,name
this is a test for lines.
------------------------
\`\`\`

### Index - add index for csv
`;
}

export function mdStr() {
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
(Select column: <u>name</u>, Number of the string: <u>3</u>)
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
(Select column: <u>name</u>, Number of the string: <u>3</u>)
\`\`\`
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    m-1     │
│  2  │ jerry-2  │    y-2     │
│  3  | hansen-3 |    n-3     │
└─────┴──────────┴────────────┘
\`\`\`

### 3. Slice
(Select column: <u>name</u>, Start index: <u>1</u>, Length of the slice: <u>3</u>)
\`\`\`
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_slice │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │   tom      │
│  2  │ jerry-2  │   jer      │
│  3  | hansen-3 |   han      │
└─────┴──────────┴────────────┘
\`\`\`
(Select column: <u>name</u>, Start index: <u>-1</u>, Length of the slice: <u>3</u>)
\`\`\`
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_slice │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │   m-1      │
│  2  │ jerry-2  │   y-2      │
│  3  | hansen-3 |   n-3      │
└─────┴──────────┴────────────┘
\`\`\`

### 4. SplitN
(Select column: <u>name</u>, nth/max number of items to return: <u>1</u>, Substring to split by: <u>-</u>)
\`\`\`
┌─────┬──────────┬──────────┐
│ idx │ name     │ name_nth │
├─────┼──────────┼──────────┤
│  1  │ tom-1    │  tom     │
│  2  │ jerry-2  │  jerry   │
│  3  | hansen-3 |  hansen  │
└─────┴──────────┴──────────┘
\`\`\`

### 5. SplitMax
(Select column: <u>name</u>, nth/max number of items to return: <u>2</u>, Substring to split by: <u>-</u>)
\`\`\`
┌─────┬──────────┬────────────┬────────────┐
│ idx │ name     │ name_nmax1 │ name_nmax2 │
├─────┼──────────┼────────────┼────────────┤
│  1  │ tom-1    │  tom       │     1      │
│  2  │ jerry-2  │  jerry     │     2      │
│  3  | hansen-3 |  hansen    │     3      │
└─────┴──────────┴────────────┴────────────┘
\`\`\`

### 6. PadLeft
(Select column: <u>idx</u>, Pad the string until it reaches this length: <u>2</u>, The character to pad the string with: <u>*</u>)
\`\`\`
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│ *1  │ tom-1    │
│ *2  │ jerry-2  │
│ *3  | hansen-3 |
└─────┴──────────┘
\`\`\`

### 7. PadRight
(Select column: <u>idx</u>, Pad the string until it reaches this length: <u>2</u>, The character to pad the string with: <u>*</u>)
\`\`\`
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│ 1*  │ tom-1    │
│ 2*  │ jerry-2  │
│ 3*  | hansen-3 |
└─────┴──────────┘
\`\`\`

### 7. PadBoth
(Select column: <u>idx</u>, Pad the string until it reaches this length: <u>3</u>, The character to pad the string with: <u>*</u>)
\`\`\`
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│ *1* │ tom-1    │
│ *2* │ jerry-2  │
│ *3* | hansen-3 |
└─────┴──────────┘
\`\`\`
`;
}

export function mdSearch() {
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
(Select column: <u>name</u>, Search mode: <u>Equal</u>, Search conditions: <u>tom|jerry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 2. EqualMulti
(Select column: <u>name</u>, Search mode: <u>EqualMulti</u>, Search conditions: <u>tom|jerry</u>)
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
(Select column: <u>name</u>, Search mode: <u>NotEqual</u>, Search conditions: <u>tom|jerry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 4. Contains
(Select column: <u>name</u>, Search mode: <u>Contains</u>, Search conditions: <u>om|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 5. ContainsMulti
(Select column: <u>name</u>, Search mode: <u>ContainsMulti</u>, Search conditions: <u>om|jer</u>)
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
(Select column: <u>name</u>, Search mode: <u>NotContains</u>, Search conditions: <u>om|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 7. StartsWith
(Select column: <u>name</u>, Search mode: <u>StartsWith</u>, Search conditions: <u>to|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 8. StartsWithMulti
(Select column: <u>name</u>, Search mode: <u>StartsWithMulti</u>, Search conditions: <u>to|jer</u>)
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
(Select column: <u>name</u>, Search mode: <u>NotStartsWith</u>, Search conditions: <u>to|jer</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 10. EndsWith
(Select column: <u>name</u>, Search mode: <u>EndsWith</u>, Search conditions: <u>om|rry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 11. EndsWithMulti
(Select column: <u>name</u>, Search mode: <u>EndsWithMulti</u>, Search conditions: <u>om|rry</u>)
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
(Select column: <u>name</u>, Search mode: <u>NotEndsWith</u>, Search conditions: <u>om|rry</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
\`\`\`

### 13. Regex
(Select column: <u>name</u>, Search mode: <u>Regex</u>, Search conditions: <u>hansen</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 14. IsNull
(Select column: <u>name</u>, Search mode: <u>IsNull</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│     │        │
└─────┴────────┘
\`\`\`

### 15. IsNotNull
(Select column: <u>name</u>, Search mode: <u>IsNotNull</u>)
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
(Select column: <u>idx</u>, Search mode: <u>gt</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 17. ge
(Select column: <u>idx</u>, Search mode: <u>ge</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 18. lt
(Select column: <u>idx</u>, Search mode: <u>lt</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘
\`\`\`

### 19. le
(Select column: <u>idx</u>, Search mode: <u>le</u>, Search conditions: <u>2</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### 20. Between
(Select column: <u>idx</u>, Search mode: <u>le</u>, Search conditions: <u>1|2</u>)
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

export function mdRename() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`
Set new headers: idx1, name1
\`\`\`
┌──────┬────────┐
│ idx1 │ name1  │
├──────┼────────┤
│  1   │ tom    │
│  2   │ jerry  │
│  3   | hansen |
└──────┴────────┘
\`\`\`
`;
}

export function mdSelect() {
  return `
\`\`\`
sample file
┌─────┬────────┬─────────────┐
│ idx │ name   │ _filename_  │
├─────┼────────┼─────────────┤
│  1  │ tom    │ test1.csv   │
│  2  │ jerry  │ test1.csv   │
│  3  | hansen | test1.csv   │
└─────┴────────┴─────────────┘
\`\`\`

### 1. Include
(Select column: <u>name,idx</u>, Select mode: <u>Include</u>)
\`\`\`
┌────────┬─────┐
│ name   │ idx │
├────────┼─────┤
│ tom    │  1  │
│ jerry  │  2  │
│ hansen |  3  |
└────────┴─────┘
\`\`\`

### 2. Exclude
(Select column: <u>name,idx</u>, Select mode: <u>Exclude</u>)
\`\`\`
┌─────────────┐
│ _filename_  │
├─────────────┤
│ test1.csv   │
│ test1.csv   │
│ test1.csv   │
└─────────────┘
\`\`\`
`;
}

export function mdFill() {
  return `
\`\`\`
sample file
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │         │
│  2  │ jerry   │
│  3  |         |
└─────┴─────────┘
\`\`\`

### 1. fill
(Select column: <u>name</u>, fill mode: <u>fill</u>, fill value: <u>jerry</u>)
\`\`\`
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │ jerry   │
│  2  │ jerry   │
│  3  | jerry   |
└─────┴─────────┘
\`\`\`

### 2. f-fill
(Select column: <u>name</u>, fill mode: <u>f-fill</u>)
\`\`\`
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │         │
│  2  │ jerry   │
│  3  | jerry   |
└─────┴─────────┘
\`\`\`
  `;
}

export function mdCount() {
  return `
### 1. Count
\`\`\`
Count result: 3
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### 2. [Index](../src-tauri/src/lib/cmd/idx.rs) - add index for csv

### 3. Check - detecting issues caused by double quotation marks
\`\`\`
Check result: 2 (This is an incorrect result)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │  tom   │
│  2  │ "jerry │
│  3  | hansen |
└─────┴────────┘
\`\`\`
  `;
}

export function mdJoin() {
  return `
\`\`\`
sample file
left.csv           right.csv
┌─────┬────────┐   ┌─────┬──────┐
│ idx │ name   │   │ idx │ age  │
├─────┼────────┤   ├─────┼──────┤
│  1  │ tom    │   │  1  │ 20   │
│  2  │ jerry  │   │  3  | 18   |
│  3  | hansen |   │  5  │ 10   |
└─────┴────────┘   └─────┴──────┘
\`\`\`

### inner
\`\`\`
inner join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  3  │ hansen │  3  | 18   |
└─────┴────────┴─────┴──────┘
\`\`\`

### left
\`\`\`
left outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  2  │ jerry  |     |      |
│  3  | hansen |  3  │ 18   |
└─────┴────────┴─────┴──────┘
\`\`\`

### right
\`\`\`
right outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  3  │ hansen │  3  | 18   |
│     |        │  5  │ 10   |
└─────┴────────┴─────┴──────┘
\`\`\`

### full
\`\`\`
full outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  2  │ jerry  │     |      |
│  3  | hansen |  3  │ 18   |
│     |        │  5  │ 10   |
└─────┴────────┴─────┴──────┘
\`\`\`

### cross
\`\`\`
cross join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  1  │ tom    │  3  │ 18   │
│  1  │ tom    │  5  │ 10   │
│  2  │ jerry  │  1  | 20   |
│  2  │ jerry  │  3  | 18   |
│  2  │ jerry  │  5  | 10   |
│  3  | hansen |  1  │ 20   |
│  3  | hansen |  3  │ 18   |
│  3  | hansen |  5  │ 10   |
└─────┴────────┴─────┴──────┘
\`\`\`

### left semi
\`\`\`
left semi join result (left_on='idx', right_on='idx')
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### left anti
\`\`\`
left anti join result (left_on='idx', right_on='idx')
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
\`\`\`

### right semi
\`\`\`
right semi join result (left_on='idx', right_on='idx')
┌─────┬──────┐
│ idx │ age  │
├─────┼──────┤
│  1  │ 20   │
│  3  | 18   |
└─────┴──────┘
\`\`\`

### right anti
\`\`\`
right anti join result (left_on='idx', right_on='idx')
┌─────┬──────┐
│ idx │ age  │
├─────┼──────┤
│  5  │ 10   |
└─────┴──────┘
\`\`\`
`;
}

export function mdSkip() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

\`\`\`
Skip result (skip rows: 2)
┌─────┬────────┐
│  2  │ jerry  │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
\`\`\`
  `;
}

export function mdEnumer() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

\`\`\`
Enumerate result
┌───────────────┬──────┬────────┐
│ enumerate_idx │ idx  │ name   │
├───────────────┼──────┼────────┤
│       1       │  1   │ tom    │
│       2       │  2   │ jerry  │
│       3       |  3   | hansen │
└───────────────┴──────┴────────┘
\`\`\`
`;
}

export function mdPinyin() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ 汤姆   │
│  2  │ 杰瑞   │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### upper
(Select column: <u>name</u>, pinyin style: <u>upper</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ TANGMU │
│  2  │ JIERUI │
│  3  | hansen |
└─────┴────────┘
\`\`\`

### lower
(Select column: <u>name</u>, pinyin style: <u>lower</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tangmu │
│  2  │ jierui │
│  3  | hansen |
└─────┴────────┘
\`\`\`
`;
}

export function mdReplace() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

(Select column: <u>name</u>, regex pattern: <u>tom</u>, replacement: <u>guy</u>)
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ guy    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`
`;
}

export function mdReverse() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

\`\`\`
Reverse result
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
│  2  │ jerry  │
│  1  | tom    |
└─────┴────────┘
\`\`\`
`;
}

export function mdSort() {
  return `
### Documents to be added...
`;
}

export function mdTranspose() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

\`\`\`
Transpose result
┌─────┬─────┬──────┬───────┐
│ idx │ 1   │ 2    │ 3     │
├─────┼─────┼──────┼───────┤
│ name│ tom │ jerry│ hansen│
└─────┴─────┴──────┴───────┘
\`\`\`
`;
}

export function mdSeparate() {
  return `
### Suggestion: set quoting to false
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom,1  │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

good file
\`\`\`
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

bad file
\`\`\`
┌─────┬────────┐
│  1  │ tom,1  │
└─────┴────────┘
\`\`\`
`;
}

export function mdSlice() {
  return `
\`\`\`
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
\`\`\`

(start: <u>1</u>, end: <u>2</u>)
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
