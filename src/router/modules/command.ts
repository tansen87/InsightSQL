export default {
  path: "/command",
  redirect: "/command/cmd",
  meta: {
    icon: "command",
    title: "command",
    rank: 1
  },
  children: [
    {
      path: "/command/cmd",
      name: "command",
      component: () => import("@/views/command/cmd.vue"),
      meta: {
        title: "command"
      }
    },
    {
      path: "/command/cat",
      name: "cat",
      component: () => import("@/views/command/cat.vue"),
      meta: {
        title: "cat",
        showLink: false
      }
    },
    {
      path: "/command/excel",
      name: "excel",
      component: () => import("@/views/command/excel.vue"),
      meta: {
        title: "Excel to csv",
        showLink: false
      }
    },
    {
      path: "/command/csv",
      name: "csv",
      component: () => import("@/views/command/csv.vue"),
      meta: {
        title: "csv to xlsx",
        showLink: false
      }
    },
    {
      path: "/command/count",
      name: "count",
      component: () => import("@/views/command/count.vue"),
      meta: {
        title: "count",
        showLink: false
      }
    },
    {
      path: "/command/rename",
      name: "rename",
      component: () => import("@/views/command/rename.vue"),
      meta: {
        title: "rename",
        showLink: false
      }
    },
    {
      path: "/command/select",
      name: "select",
      component: () => import("@/views/command/select.vue"),
      meta: {
        title: "select",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
