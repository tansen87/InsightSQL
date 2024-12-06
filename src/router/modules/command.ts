export default {
  path: "/command",
  redirect: "/command/index",
  meta: {
    icon: "command",
    title: "command",
    rank: 1
  },
  children: [
    {
      path: "/command/index",
      name: "command",
      component: () => import("@/views/command/index.vue"),
      meta: {
        title: "command"
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
      path: "/command/components/excel",
      name: "excel",
      component: () => import("@/views/command/components/excel.vue"),
      meta: {
        title: "Excel to csv",
        showLink: false
      }
    },
    {
      path: "/command/components/csv",
      name: "csv",
      component: () => import("@/views/command/components/csv.vue"),
      meta: {
        title: "csv to xlsx",
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
      path: "/command/components/access",
      name: "access",
      component: () => import("@/views/command/components/access.vue"),
      meta: {
        title: "access to csv",
        showLink: false
      }
    },
    {
      path: "/command/components/dbf",
      name: "dbf",
      component: () => import("@/views/command/components/dbf.vue"),
      meta: {
        title: "dbf to csv",
        showLink: false
      }
    },
    {
      path: "/command/components/behead",
      name: "behead",
      component: () => import("@/views/command/components/behead.vue"),
      meta: {
        title: "drop headers",
        showLink: false
      }
    },
    {
      path: "/command/components/offset",
      name: "offset",
      component: () => import("@/views/command/components/offset.vue"),
      meta: {
        title: "offset",
        showLink: false
      }
    },
    {
      path: "/command/components/addIndex",
      name: "addIndex",
      component: () => import("@/views/command/components/addIndex.vue"),
      meta: {
        title: "addIndex",
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
    }
  ]
} satisfies RouteConfigsTable;
