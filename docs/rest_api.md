# atom rest api

atom 接口文档

服务器的地址：`http://<BIND_ADDRESS>`，其中 `BIND_ADDRESS` 来自 `.env`。下面文档用了本地开发地址： `localhost:7070`

> 没有配置 https ，浏览器服务正常访问；可以用 `curl` 或 `http` 命令行工具测试。

## users

从老到新的获取所有 users，通过该接口构建本地数据库。

> API: `/users`

params:

- offset, 从 **零** 开始；默认是 `0`
- limit，每次返回多少条，**最大为100**；默认是`20`
- topic, topic 地址

注：根据 user allow/deny topic 被抓到的时间的顺序返回，先返回抓到最旧的数据。

发送请求

    curl -s 'localhost:7070/users?topic=a7b751cc0e2f6c5be01ce95bc80b02d071022af4&offset=0&limit=2' | python -m json.tool
    [
        {
            "user_address": "74fb01e4d7ea240560978d98f66136c6211d3d61",
            "status": "allow",
            "tx_id": "04a90b96ba3d27b4ed2872f1eb3ad4bdd5c85178c6ddd9363ef8e6be62807a04",
            "updated_at": "2019-12-26T03:46:58.032960",
            "topic": "a7b751cc0e2f6c5be01ce95bc80b02d071022af4"
        }
    ]

## all posts

从老到新的获取所有 posts，不管 `user allow/deny topic`

通过该接口构建本地数据库，结合 `users` 决定如何一些被 `deny user` 的文章是否展示。 

> API: `/posts`

params:

- offset, 从 **零** 开始；默认是零
- limit，每次返回多少条，**最大为100**；默认是`20`
- topic, topic 地址

注：

- 根据 posts.updated_at 的顺序返回，先返回抓到最旧的数据
- 返回 xml ，和之前 scp 同步过去的 xml 格式相同

发送请求

    $ curl 'localhost:7070/posts?topic=a7b751cc0e2f6c5be01ce95bc80b02d071022af4&offset=0&limit=2'
    # 返回的 xml 太长就不粘贴到这里了

## latest posts

结合 `users allow/deny`，从新到久的获取所有 posts。

reader 可以不用自己构建本地数据库，直接调用该接口，获取数据并展示。

> API: `/atom`

params:

- offset, 从 **零** 开始；默认是零
- limit，每次返回多少条，**最大为100**；默认是`20`
- topic, topic 地址

注：

- 根据 posts.updated_at 的倒序返回，先返回最后上链的 post。
- 返回 xml ，和之前 scp 同步过去的 xml 格式相同
- 因为先返回最新上链的数据，所以，展示时要考虑是否返回的数据是否已经被展示过，做去重工作；并决定是否继续调用获取新数据

发送请求

    $ curl 'localhost:7070/atom?topic=a7b751cc0e2f6c5be01ce95bc80b02d071022af4&offset=0&limit=2'
    # 返回的 xml 太长就不粘贴到这里了
