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

  const manualHighlight = async () => {
    await nextTick();
    Prism.highlightAll();
  };

  return {
    markdownContent,
    compiledMarkdown,
    manualHighlight,
    updateMarkdownContent: (newMarkdownFn: () => string) => {
      markdownContent.value = newMarkdownFn();
    }
  };
}

export function catContent() {
  return `
### What happens when running cat
For example, let's say we have two file **data1.csv**, **data2.csv** with the following contents:

**data1.csv**
\`\`\`sql
name,age,gender
Tom,18,male
Jerry,19,male
\`\`\`

**data2.csv**
\`\`\`sql
age,gender,name
4,male,Patrick
24,female,Sandy
\`\`\`

Then, we will receive the following files and with the following contents:

**cat_xxx.csv**
\`\`\`sql
name,age,gender,FileName
Tom,18,male,data1.csv
Jerry,19,male,data1.csv
Patrick,4,male,data2.csv
Sandy,24,female,data2.csv
\`\`\`
`;
}

export function splitContent() {
  return `
### What happens when running split

For example, let's say we have a file **data.csv** with the following contents:

\`\`\`sql
name,age,gender
Tom,18,male
Jerry,19,male
Patrick,4,male
Sandy,24,female
\`\`\`

Then, we set the **Split rows** to 3, we will receive the following two files and with the following contents:

**data.split_0.csv**

\`\`\`sql
name,age,gender
Tom,18,male
Jerry,19,male
Patrick,4,male
\`\`\`

**data.split_1.csv**

\`\`\`sql
name,age,gender
Sandy,24,female
\`\`\`
`;
}

export function sliceContent() {
  return `
### What happens when running slice

For example, let's say we have a file (**data.csv**) with the following contents:

\`\`\`sql
name,age,gender
汤-姆-1,18,男
杰-瑞-2,19,male
Sa-n-dy,24,female
\`\`\`

1st, we set **Slice by column** to <u>name</u> and set the **Numer of slice** to <u>2</u>
and set the **Slice mode** to <u>Left</u>,
we will receive the following files and with the following contents:

left mode: **data.slice.csv**

\`\`\`sql
name,age,gender,name_nchar
汤-姆-1,18,男,汤-
杰-瑞-2,19,male,杰-
Sa-n-dy,24,female,Sa
\`\`\`

2nd, we set **Slice by column** to <u>name</u> and set the **Numer of slice** to <u>2</u>
and set the **Slice mode** to <u>Right</u>,
we will receive the following files and with the following contents:

right mode: **data.slice.csv**

\`\`\`sql
name,age,gender,name_nchar
汤-姆-1,18,男,-1
杰-瑞-2,19,male,-2
Sa-n-dy,24,female,dy
\`\`\`

3rd, we set **Slice by column** to <u>name</u> and set the **Numer of slice** to <u>2</u>
and set the **Slice separator** to <u>-</u> and set the **Slice mode** to <u>Nth</u>,
we will receive the following files and with the following contents:

nth mode: **data.slice.csv**

\`\`\`sql
name,age,gender,name_nth
汤-姆-1,18,男,姆
杰-瑞-2,19,male,瑞
Sa-n-dy,24,female,n
\`\`\`

4th, we set **Slice by column** to <u>name</u> and set the **Numer of slice** to <u>2</u>
and set the **Slice separator** to <u>-</u> and set the **Slice mode** to <u>Nmax</u>,
we will receive the following files and with the following contents:

nmax mode: **data.slice.csv**

\`\`\`sql
name,age,gender,name_nmax1,name_nmax2
汤-姆-1,18,男,汤,姆
杰-瑞-2,19,male,杰,瑞
Sa-n-dy,24,female,Sa,n
\`\`\`
`;
}
