CKB中可以获取到Inputs中Live Cell的创建时间，但是无法获取当前交易所在区块的时间戳。当需要获取时间信息时，可以使用两步机制，例如在去中心化借贷场景中，存款方在提取UDT时，先生成取款单，然后再进行取款操作，根据取款单的时间计算利息。而对于还款的情况会更复杂，要考虑用户提前提交还款单后，可能存在迟迟不确认还款的情况，最终还款时利息会少很多，需要设计一种复杂机制来处理这种情况。**两步机制会造成中间状态的问题**。

在这里通过使用时间Oracle的方法来获得当前的时间，避免这种两步操作，降低合约的复杂度。

时间Oracle可以一小时更新一次或者10分钟等更新一次。实时性要求，跟时间Oralce的更新频率有关。

结构如下：

```
data:
    
type:
    code_hash: time oracle type script
    args: type_id byte32, always_success script hash bytes32
lock:
    always_success
```

允许任意用户去把这个Cell 作为Input，并输出Output。

然后其他人引用最新的Live cell作为dep，可以找到这个live cell的时间戳作为最新的时间。

这种方式来提供最新的时间。

这里有几个验证条件：

1）Cell可以被创建

2）Cell可以被更新

3）Cell不可以被销毁

4）Output Cell的锁必须是always_success

这里希望只有一个合法的Live 时间Oracle cell，但是TypeID是只能限制合约脚本的更新，无法增加的自己的逻辑。这里使用Rust自定义TypeID的方式，https://github.com/axonweb3/ckb-type-id

由于任何人可以更新时间Oracle，所以时间Oracle可以存在多个维护方，同时时间信息来自于新的Live cell所在区块头。这样避免了单一维护方停更的风险，以及时间数据作恶的风险。

在使用时，先更新时间Oralce，然后引用最新的时间Oralce cell 即可。

对于一些需要使用时间Oracle的 Dapp服务方，可以设置定时服务去更新时间Oracle，这样就避免其用户引用错误的时间。

这个设计有一个潜在的风险，就是攻击者可以非常频繁的更新时间Oracle，这样其他人就始终难拿到live cell，规避这种方式的办法之一是设置消耗的ckb，比如最少消耗多少ckb，相当于烧毁掉一部分ckb，这种方式可以增加攻击的成本，但是无法完全避免。

源码：https://github.com/u2/ckb-time-oracle

已经部署在测试网

```
const time_type_id = "0x63eb41aadea32411547cdd9b62f7347b3c719cd1ae17f28123765d9098af7c96";

const time_cell_dep =  {
    outPoint: { txHash: "0x46093a1cbe8657478bccbc5f664c075fd7dfac961bbc754c660f65922fadecaf", index: 0 },
    depType: 0,
}

const time_script = {
  codeHash: "0x63eb41aadea32411547cdd9b62f7347b3c719cd1ae17f28123765d9098af7c96",
  hashType: "type",
  args: "0x7ac202f08b41448259ca387eee84ac4c285a33124628841ad7ee1815c61ec49108d1374b76cb5104ace0e2394b5d873f05c7ed8e8659d54b4cc29a98bf66b820";
};
```
