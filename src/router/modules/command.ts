export default {
  path: "/command",
  redirect: "/command/index",
  meta: {
    icon: "command",
    title: "cmd",
    rank: 1
  },
  children: [
    {
      path: "/command/index",
      name: "cmd",
      component: () => import("@/views/command/index.vue"),
      meta: {
        title: "cmd"
      }
    },
    {
      path: "/command/components/idx",
      name: "csv_idx",
      component: () => import("@/views/command/components/idx.vue"),
      meta: {
        title: "csv_idx",
        showLink: false
      }
    },
    {
      path: "/command/components/apply",
      name: "apply",
      component: () => import("@/views/command/components/apply.vue"),
      meta: {
        title: "apply",
        showLink: false
      }
    },
    {
      path: "/command/components/cat",
      name: "cat",
      component: () => import("@/views/command/components/cat.vue"),
      meta: {
        title: "cat",
        showLink: false
      }
    },
    {
      path: "/command/components/convert",
      name: "convert",
      component: () => import("@/views/command/components/convert.vue"),
      meta: {
        title: "convert",
        showLink: false
      }
    },
    {
      path: "/command/components/count",
      name: "count",
      component: () => import("@/views/command/components/count.vue"),
      meta: {
        title: "count",
        showLink: false
      }
    },
    {
      path: "/command/components/rename",
      name: "rename",
      component: () => import("@/views/command/components/rename.vue"),
      meta: {
        title: "rename",
        showLink: false
      }
    },
    {
      path: "/command/components/select",
      name: "select",
      component: () => import("@/views/command/components/select.vue"),
      meta: {
        title: "select",
        showLink: false
      }
    },
    {
      path: "/command/components/separate",
      name: "separate",
      component: () => import("@/views/command/components/separate.vue"),
      meta: {
        title: "separate",
        showLink: false
      }
    },
    {
      path: "/command/components/search",
      name: "search",
      component: () => import("@/views/command/components/search.vue"),
      meta: {
        title: "search",
        showLink: false
      }
    },
    {
      path: "/command/components/fill",
      name: "fill",
      component: () => import("@/views/command/components/fill.vue"),
      meta: {
        title: "fill",
        showLink: false
      }
    },
    {
      path: "/command/components/split",
      name: "split",
      component: () => import("@/views/command/components/split.vue"),
      meta: {
        title: "split",
        showLink: false
      }
    },
    {
      path: "/command/components/skip",
      name: "skip",
      component: () => import("@/views/command/components/skip.vue"),
      meta: {
        title: "skip",
        showLink: false
      }
    },
    {
      path: "/command/components/slice",
      name: "slice",
      component: () => import("@/views/command/components/slice.vue"),
      meta: {
        title: "slice",
        showLink: false
      }
    },
    {
      path: "/command/components/enumerate",
      name: "enumerate",
      component: () => import("@/views/command/components/enumerate.vue"),
      meta: {
        title: "enumerate",
        showLink: false
      }
    },
    {
      path: "/command/components/pinyin",
      name: "pinyin",
      component: () => import("@/views/command/components/pinyin.vue"),
      meta: {
        title: "chinese to pinyin",
        showLink: false
      }
    },
    {
      path: "/command/components/replace",
      name: "replace",
      component: () => import("@/views/command/components/replace.vue"),
      meta: {
        title: "replace",
        showLink: false
      }
    },
    {
      path: "/command/components/join",
      name: "join",
      component: () => import("@/views/command/components/join.vue"),
      meta: {
        title: "join",
        showLink: false
      }
    },
    {
      path: "/command/components/sort",
      name: "sort",
      component: () => import("@/views/command/components/sort.vue"),
      meta: {
        title: "sort",
        showLink: false
      }
    },
    {
      path: "/command/components/string",
      name: "string",
      component: () => import("@/views/command/components/string.vue"),
      meta: {
        title: "string",
        showLink: false
      }
    },
    {
      path: "/command/components/reverse",
      name: "reverse",
      component: () => import("@/views/command/components/reverse.vue"),
      meta: {
        title: "reverse",
        showLink: false
      }
    },
    {
      path: "/command/components/transpose",
      name: "transpose",
      component: () => import("@/views/command/components/transpose.vue"),
      meta: {
        title: "transpose",
        showLink: false
      }
    },
    {
      path: "/command/components/insert",
      name: "insert",
      component: () => import("@/views/command/components/insert.vue"),
      meta: {
        title: "insert",
        showLink: false
      }
    },
    {
      path: "/command/components/datefmt",
      name: "datefmt",
      component: () => import("@/views/command/components/datefmt.vue"),
      meta: {
        title: "datefmt",
        showLink: false
      }
    },
    {
      path: "/command/components/traverse",
      name: "traverse",
      component: () => import("@/views/command/components/traverse.vue"),
      meta: {
        title: "traverse directory",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
