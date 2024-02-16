export default {
  path: "/error",
  redirect: "/error/md",
  meta: {
    icon: "informationLine",
    title: "other",
    // showLink: false,
    rank: 9
  },
  children: [
    {
      path: "/error/md",
      name: "md",
      component: () => import("@/views/error/md.vue"),
      meta: {
        title: "markdown"
      }
    },
    {
      path: "/error/404",
      name: "404",
      component: () => import("@/views/error/404.vue"),
      meta: {
        title: "404"
      }
    },
    {
      path: "/error/500",
      name: "500",
      component: () => import("@/views/error/500.vue"),
      meta: {
        title: "500"
      }
    }
  ]
} as RouteConfigsTable;
