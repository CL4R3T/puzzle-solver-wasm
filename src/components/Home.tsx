import './Home.css'

export default function Home() {
  return (
    <div className="app">
      <header className="app-header">
        <h1>CLRT 的纸笔求解器</h1>
      </header>
      <main className="app-main home-main">
        <p className="home-text">
          这是一个通用的谜题求解 Web 应用，目前支持数独求解功能。
          您可以在左侧导航栏中选择不同的谜题类型进行求解。
        </p>
        <p className="home-text">
          更多谜题类型正在开发中，敬请期待...
        </p>
      </main>
    </div>
  )
}