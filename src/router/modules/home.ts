const { VITE_HIDE_HOME } = import.meta.env;
const Layout = () => import("@/layout/index.vue");

export default {
  path: "/",
  name: "Home",
  component: Layout,
  redirect: "/queries",
  meta: {
    icon: "homeFilled",
    title: "sqlp",
    rank: 0
  },
  children: [
    {
      path: "/queries",
      name: "Welcome",
      component: () => import("@/views/queries/index.vue"),
      meta: {
        title: "sqlp",
        showLink: VITE_HIDE_HOME === "true" ? false : true
      }
    }
  ]
} as RouteConfigsTable;
