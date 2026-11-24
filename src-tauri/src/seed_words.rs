//! M9 词书与种子词库。
//!
//! 词书按学段 / 考试划分（小学、初中、高中、大学、雅思、专业），每个词书带一个
//! 难度系数（difficulty）：难度越高，SRS 首次复习间隔越短（更频繁巩固）。
//! 每个单词还可带内在难度（intrinsic difficulty，默认 1.0）做细粒度调整；
//! 学习新词时有效难度 = 词书难度 × 单词内在难度，快照写入 `word_states.difficulty`。
//! 迁移 1 已存在的 20 个种子单词会按难度映射进对应词书，保持数据唯一约束。

pub struct SeedWord {
    pub word: &'static str,
    pub phonetic: &'static str,
    pub meaning: &'static str,
    pub example_en: &'static str,
    pub example_zh: &'static str,
    pub difficulty: f64,
}

pub struct SeedBook {
    pub code: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub level: &'static str,
    pub difficulty: f64,
    pub sort_order: i64,
    pub words: &'static [SeedWord],
}

/// 内在难度 1.0（常规单词）。
const fn w(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.0,
    }
}

/// 内在难度 1.25（较难单词）。
const fn hw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.25,
    }
}

/// 内在难度 1.5（专业长词等最难单词）。
const fn vw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.5,
    }
}

/// 内在难度 0.6（基础常见词，最容易）。
const fn ew(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 0.6,
    }
}

/// 内在难度 0.8（较容易词）。
const fn lw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 0.8,
    }
}

/// 内在难度 0.9（略容易词）。
const fn sw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 0.9,
    }
}

/// 内在难度 1.1（略难词）。
const fn nw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.1,
    }
}

/// 内在难度 1.2（较难词）。
const fn dw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.2,
    }
}

/// 内在难度 1.4（很难词）。
const fn tw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.4,
    }
}

const PRIMARY_WORDS: &[SeedWord] = &[
    w("apple", "/ˈæpl/", "苹果", "I eat an apple every day.", "我每天吃一个苹果。"),
    lw("banana", "/bəˈnɑːnə/", "香蕉", "The monkey likes bananas.", "猴子喜欢香蕉。"),
    ew("cat", "/kæt/", "猫", "The cat is sleeping on the sofa.", "猫在沙发上睡觉。"),
    ew("dog", "/dɒɡ/", "狗", "My dog likes to run in the park.", "我的狗喜欢在公园里跑。"),
    ew("egg", "/eɡ/", "鸡蛋", "We have eggs for breakfast.", "我们早餐吃鸡蛋。"),
    ew("fish", "/fɪʃ/", "鱼", "Fish can swim very fast.", "鱼游得很快。"),
    lw("girl", "/ɡɜːl/", "女孩", "The girl is reading a book.", "那个女孩正在读书。"),
    lw("house", "/haʊs/", "房子", "Our house has a small garden.", "我们的房子有一个小花园。"),
    ew("ice", "/aɪs/", "冰", "Ice melts in the sun.", "冰在阳光下融化。"),
    lw("juice", "/dʒuːs/", "果汁", "I drink orange juice in the morning.", "我早上喝橙汁。"),
    lw("kite", "/kaɪt/", "风筝", "The boy flies a kite in the wind.", "男孩在风中放风筝。"),
    lw("lion", "/ˈlaɪən/", "狮子", "The lion is the king of the jungle.", "狮子是丛林之王。"),
    ew("milk", "/mɪlk/", "牛奶", "Milk is good for your bones.", "牛奶对你的骨骼有好处。"),
    ew("nose", "/nəʊz/", "鼻子", "Your nose helps you smell flowers.", "鼻子帮你闻花香。"),
    lw("orange", "/ˈɒrɪndʒ/", "橙子；橙色", "The orange is sweet and juicy.", "这个橙子又甜又多汁。"),
    ew("pen", "/pen/", "钢笔", "May I borrow your pen?", "我可以借你的钢笔吗？"),
    lw("queen", "/kwiːn/", "女王", "The queen lives in a big palace.", "女王住在一座大宫殿里。"),
    ew("rain", "/reɪn/", "雨", "The rain makes the grass grow.", "雨水让草生长。"),
    ew("sun", "/sʌn/", "太阳", "The sun rises in the east.", "太阳从东方升起。"),
    ew("tree", "/triː/", "树", "Birds build nests in the tree.", "鸟儿在树上筑巢。"),
    lw("umbrella", "/ʌmˈbrelə/", "雨伞", "Take an umbrella when it rains.", "下雨时要带伞。"),
    lw("violin", "/ˌvaɪəˈlɪn/", "小提琴", "She plays the violin beautifully.", "她小提琴拉得很好听。"),
    ew("water", "/ˈwɔːtə/", "水", "Please drink more water every day.", "请每天多喝水。"),
    ew("yellow", "/ˈjeləʊ/", "黄色", "The yellow flowers are beautiful.", "黄色的花很漂亮。"),
    ew("zoo", "/zuː/", "动物园", "We saw pandas at the zoo.", "我们在动物园看到了熊猫。"),
    lw("friend", "/frend/", "朋友", "A good friend is always kind.", "好朋友总是很友善。"),
    lw("family", "/ˈfæməli/", "家庭", "My family has four people.", "我家有四口人。"),
    lw("teacher", "/ˈtiːtʃə/", "老师", "Our teacher explains everything clearly.", "我们的老师把一切都讲得很清楚。"),
    // 数字与时间
    ew("one", "/wʌn/", "一", "I have one apple.", "我有一个苹果。"),
    ew("two", "/tuː/", "二", "I have two hands.", "我有两只手。"),
    ew("three", "/θriː/", "三", "There are three birds in the tree.", "树上有三只鸟。"),
    ew("four", "/fɔː/", "四", "We have four seasons.", "我们有四个季节。"),
    ew("five", "/faɪv/", "五", "I can count to five.", "我能数到五。"),
    ew("six", "/sɪks/", "六", "There are six chairs in the room.", "房间里有六把椅子。"),
    ew("seven", "/ˈsevən/", "七", "A week has seven days.", "一周有七天。"),
    ew("eight", "/eɪt/", "八", "I get up at eight.", "我八点起床。"),
    ew("nine", "/naɪn/", "九", "Nine plus one is ten.", "九加一等于十。"),
    ew("ten", "/ten/", "十", "I am ten years old.", "我十岁了。"),
    ew("clock", "/klɒk/", "时钟", "The clock on the wall is new.", "墙上的时钟是新的。"),
    ew("day", "/deɪ/", "天；白天", "Today is a sunny day.", "今天是个晴天。"),
    lw("week", "/wiːk/", "周", "I go swimming once a week.", "我每周去游一次泳。"),
    lw("month", "/mʌnθ/", "月", "June is a warm month.", "六月是温暖的月份。"),
    lw("year", "/jɪə/", "年", "A year has twelve months.", "一年有十二个月。"),
    ew("morning", "/ˈmɔːnɪŋ/", "早晨", "I read books in the morning.", "我早上读书。"),
    lw("afternoon", "/ˌɑːftəˈnuːn/", "下午", "We play football in the afternoon.", "我们下午踢足球。"),
    ew("evening", "/ˈiːvnɪŋ/", "傍晚", "We watch TV in the evening.", "我们晚上看电视。"),
    ew("night", "/naɪt/", "夜晚", "The stars shine at night.", "星星在夜晚闪烁。"),
    ew("today", "/təˈdeɪ/", "今天", "Today is my birthday.", "今天是我的生日。"),
    lw("tomorrow", "/təˈmɒrəʊ/", "明天", "I will see you tomorrow.", "我明天见你。"),
    lw("spring", "/sprɪŋ/", "春天", "Flowers bloom in spring.", "花在春天开放。"),
    lw("summer", "/ˈsʌmə/", "夏天", "We swim a lot in summer.", "我们夏天经常游泳。"),
    sw("autumn", "/ˈɔːtəm/", "秋天", "Leaves fall in autumn.", "树叶在秋天落下。"),
    lw("winter", "/ˈwɪntə/", "冬天", "It snows in winter.", "冬天会下雪。"),
    // 颜色
    ew("red", "/red/", "红色", "I like the red balloon.", "我喜欢红色的气球。"),
    ew("blue", "/bluː/", "蓝色", "The sky is blue.", "天空是蓝色的。"),
    ew("green", "/ɡriːn/", "绿色", "The grass is green.", "草是绿色的。"),
    ew("black", "/blæk/", "黑色", "The cat has black fur.", "这只猫有黑色的毛。"),
    ew("white", "/waɪt/", "白色", "The snow is white.", "雪是白色的。"),
    ew("pink", "/pɪŋk/", "粉色", "She wears a pink dress.", "她穿着粉色的连衣裙。"),
    sw("purple", "/ˈpɜːpl/", "紫色", "I love purple grapes.", "我喜欢紫色的葡萄。"),
    sw("brown", "/braʊn/", "棕色", "The bear is brown.", "这只熊是棕色的。"),
    sw("grey", "/ɡreɪ/", "灰色", "The elephant is grey.", "大象是灰色的。"),
    // 动物
    ew("bird", "/bɜːd/", "鸟", "A bird sings in the tree.", "一只鸟在树上唱歌。"),
    ew("duck", "/dʌk/", "鸭子", "The duck swims on the lake.", "鸭子在湖里游泳。"),
    ew("cow", "/kaʊ/", "奶牛", "The cow gives us milk.", "奶牛给我们牛奶。"),
    ew("horse", "/hɔːs/", "马", "The horse runs very fast.", "马跑得非常快。"),
    ew("pig", "/pɪɡ/", "猪", "The pig likes to sleep.", "猪喜欢睡觉。"),
    ew("sheep", "/ʃiːp/", "绵羊", "The sheep are eating grass.", "绵羊在吃草。"),
    ew("rabbit", "/ˈræbɪt/", "兔子", "The rabbit has long ears.", "兔子有长长的耳朵。"),
    ew("monkey", "/ˈmʌŋki/", "猴子", "The monkey climbs the tree.", "猴子爬树。"),
    lw("elephant", "/ˈelɪfənt/", "大象", "The elephant has a long nose.", "大象有长长的鼻子。"),
    lw("tiger", "/ˈtaɪɡə/", "老虎", "The tiger is a big cat.", "老虎是一种大型猫科动物。"),
    ew("bear", "/beə/", "熊", "The bear eats honey.", "熊吃蜂蜜。"),
    ew("panda", "/ˈpændə/", "熊猫", "Pandas live in China.", "熊猫生活在中国。"),
    ew("frog", "/frɒɡ/", "青蛙", "The frog jumps into the pond.", "青蛙跳进池塘。"),
    ew("bee", "/biː/", "蜜蜂", "The bee makes honey.", "蜜蜂酿蜜。"),
    // 食物与饮料
    ew("rice", "/raɪs/", "米饭", "We eat rice with vegetables.", "我们吃米饭配蔬菜。"),
    ew("bread", "/bred/", "面包", "I have bread and milk for breakfast.", "我早餐吃面包喝牛奶。"),
    ew("cake", "/keɪk/", "蛋糕", "Mom made a birthday cake.", "妈妈做了一个生日蛋糕。"),
    lw("candy", "/ˈkændi/", "糖果", "Too much candy is bad for teeth.", "吃太多糖果对牙齿不好。"),
    ew("tea", "/tiː/", "茶", "Dad drinks tea after dinner.", "爸爸晚饭后喝茶。"),
    lw("coffee", "/ˈkɒfi/", "咖啡", "Mom drinks coffee in the morning.", "妈妈早上喝咖啡。"),
    ew("soup", "/suːp/", "汤", "The soup is hot.", "汤很烫。"),
    ew("meat", "/miːt/", "肉", "We eat meat every week.", "我们每周吃肉。"),
    sw("hamburger", "/ˈhæmbɜːɡə/", "汉堡包", "I had a hamburger for lunch.", "我午餐吃了一个汉堡包。"),
    sw("noodles", "/ˈnuːdlz/", "面条", "I like noodles with eggs.", "我喜欢鸡蛋面。"),
    sw("dumpling", "/ˈdʌmplɪŋ/", "饺子", "We eat dumplings on New Year's Day.", "我们在新年吃饺子。"),
    sw("vegetable", "/ˈvedʒtəbl/", "蔬菜", "Vegetables are good for health.", "蔬菜对健康有好处。"),
    lw("tomato", "/təˈmɑːtəʊ/", "西红柿", "The tomato is red and round.", "西红柿又红又圆。"),
    lw("potato", "/pəˈteɪtəʊ/", "土豆", "Potatoes grow in the ground.", "土豆长在土里。"),
    lw("carrot", "/ˈkærət/", "胡萝卜", "Rabbits love carrots.", "兔子喜欢胡萝卜。"),
    sw("watermelon", "/ˈwɔːtəmelən/", "西瓜", "The watermelon is sweet in summer.", "夏天的西瓜很甜。"),
    lw("grape", "/ɡreɪp/", "葡萄", "These grapes are sour.", "这些葡萄是酸的。"),
    ew("pear", "/peə/", "梨", "The pear is juicy.", "这个梨很多汁。"),
    lw("peach", "/piːtʃ/", "桃子", "The peach smells sweet.", "这个桃子闻起来很香。"),
    sw("strawberry", "/ˈstrɔːbəri/", "草莓", "Strawberries are red and sweet.", "草莓又红又甜。"),
    // 学校与学习
    ew("school", "/skuːl/", "学校", "I go to school by bike.", "我骑自行车上学。"),
    ew("class", "/klɑːs/", "班级；课", "We have six classes a day.", "我们每天有六节课。"),
    lw("classroom", "/ˈklɑːsruːm/", "教室", "Our classroom is clean.", "我们的教室很干净。"),
    ew("desk", "/desk/", "书桌", "My desk is near the window.", "我的书桌在窗户旁边。"),
    ew("chair", "/tʃeə/", "椅子", "Please sit on the chair.", "请坐在椅子上。"),
    ew("book", "/bʊk/", "书", "This book is interesting.", "这本书很有趣。"),
    lw("schoolbag", "/ˈskuːlbæɡ/", "书包", "My schoolbag is heavy.", "我的书包很重。"),
    lw("ruler", "/ˈruːlə/", "尺子", "I draw a line with a ruler.", "我用尺子画一条线。"),
    ew("pencil", "/ˈpensəl/", "铅笔", "I write with a pencil.", "我用铅笔写字。"),
    sw("eraser", "/ɪˈreɪzə/", "橡皮", "I use an eraser to clean mistakes.", "我用橡皮擦掉错误。"),
    sw("blackboard", "/ˈblækbɔːd/", "黑板", "The teacher writes on the blackboard.", "老师在黑板上写字。"),
    lw("homework", "/ˈhəʊmwɜːk/", "家庭作业", "I do my homework after dinner.", "我晚饭后做家庭作业。"),
    lw("lesson", "/ˈlesən/", "课", "Our first lesson is English.", "我们的第一节课是英语。"),
    lw("student", "/ˈstjuːdənt/", "学生", "The student reads every day.", "这个学生每天读书。"),
    lw("math", "/mæθ/", "数学", "Math is my favorite subject.", "数学是我最喜欢的科目。"),
    lw("music", "/ˈmjuːzɪk/", "音乐", "We sing songs in music class.", "我们在音乐课上唱歌。"),
    ew("art", "/ɑːt/", "美术；艺术", "I like art and drawing.", "我喜欢美术和画画。"),
    lw("English", "/ˈɪŋɡlɪʃ/", "英语", "We speak English in class.", "我们在课堂上说英语。"),
    sw("Chinese", "/ˌtʃaɪˈniːz/", "中文；语文", "Our Chinese teacher is kind.", "我们的语文老师很和蔼。"),
    // 家庭与人
    lw("father", "/ˈfɑːðə/", "父亲", "My father works in a bank.", "我父亲在银行工作。"),
    lw("mother", "/ˈmʌðə/", "母亲", "My mother cooks delicious food.", "我母亲做美味的食物。"),
    lw("brother", "/ˈbrʌðə/", "兄弟", "My brother is two years older than me.", "我哥哥大我两岁。"),
    lw("sister", "/ˈsɪstə/", "姐妹", "My sister likes to dance.", "我妹妹喜欢跳舞。"),
    ew("baby", "/ˈbeɪbi/", "婴儿", "The baby is sleeping.", "婴儿在睡觉。"),
    sw("grandfather", "/ˈɡrændfɑːðə/", "祖父", "My grandfather tells me stories.", "我爷爷给我讲故事。"),
    sw("grandmother", "/ˈɡrændmʌðə/", "祖母", "My grandmother grows flowers.", "我奶奶种花。"),
    ew("boy", "/bɔɪ/", "男孩", "The boy is playing football.", "男孩在踢足球。"),
    ew("man", "/mæn/", "男人", "The man is reading a newspaper.", "那个男人在看报纸。"),
    ew("woman", "/ˈwʊmən/", "女人", "The woman is a doctor.", "那个女士是医生。"),
    lw("child", "/tʃaɪld/", "孩子", "Every child likes games.", "每个孩子都喜欢游戏。"),
    // 身体
    ew("head", "/hed/", "头", "I wash my head every day.", "我每天洗头。"),
    ew("eye", "/aɪ/", "眼睛", "She has big eyes.", "她有一双大眼睛。"),
    ew("ear", "/ɪə/", "耳朵", "An elephant has big ears.", "大象有大耳朵。"),
    ew("mouth", "/maʊθ/", "嘴", "Open your mouth, please.", "请张开嘴。"),
    ew("hand", "/hænd/", "手", "We shake hands when we meet.", "我们见面时握手。"),
    ew("foot", "/fʊt/", "脚", "My foot hurts.", "我的脚疼。"),
    ew("leg", "/leɡ/", "腿", "The dog has four legs.", "狗有四条腿。"),
    ew("arm", "/ɑːm/", "手臂", "He holds a book under his arm.", "他手臂下夹着一本书。"),
    ew("face", "/feɪs/", "脸", "Her face is red.", "她的脸红了。"),
    ew("hair", "/heə/", "头发", "Her hair is long and black.", "她的头发又长又黑。"),
    ew("tooth", "/tuːθ/", "牙齿", "I brush my teeth twice a day.", "我每天刷两次牙。"),
    // 天气与自然
    ew("snow", "/snəʊ/", "雪", "The children play in the snow.", "孩子们在雪地里玩。"),
    ew("wind", "/wɪnd/", "风", "The wind blows the leaves.", "风吹动树叶。"),
    ew("cloud", "/klaʊd/", "云", "The cloud looks like a sheep.", "那朵云看起来像一只羊。"),
    ew("sky", "/skaɪ/", "天空", "The sky is clear today.", "今天天空晴朗。"),
    ew("moon", "/muːn/", "月亮", "The moon is bright tonight.", "今晚月亮很亮。"),
    ew("star", "/stɑː/", "星星", "There are many stars in the sky.", "天空中有很多星星。"),
    ew("sea", "/siː/", "大海", "We saw a boat on the sea.", "我们看到海上有一艘船。"),
    lw("river", "/ˈrɪvə/", "河流", "The river is clean and blue.", "这条河又清又蓝。"),
    lw("mountain", "/ˈmaʊntɪn/", "山", "The mountain is very tall.", "这座山非常高。"),
    lw("hill", "/hɪl/", "小山", "We ran up the hill.", "我们跑上小山。"),
    ew("flower", "/ˈflaʊə/", "花", "I pick a flower for Mom.", "我给妈妈摘了一朵花。"),
    ew("grass", "/ɡrɑːs/", "草", "The cows eat grass.", "奶牛吃草。"),
    ew("leaf", "/liːf/", "树叶", "A leaf fell from the tree.", "一片树叶从树上落下。"),
    // 衣物
    ew("shirt", "/ʃɜːt/", "衬衫", "He wears a blue shirt.", "他穿着一件蓝色衬衫。"),
    ew("dress", "/dres/", "连衣裙", "The girl in the dress is my sister.", "穿连衣裙的女孩是我妹妹。"),
    ew("coat", "/kəʊt/", "外套", "Put on your coat. It is cold.", "穿上外套，天气很冷。"),
    ew("hat", "/hæt/", "帽子", "I wear a hat in summer.", "我夏天戴帽子。"),
    ew("shoes", "/ʃuːz/", "鞋子", "My shoes are new.", "我的鞋子是新的。"),
    ew("socks", "/sɒks/", "袜子", "I wear socks to keep warm.", "我穿袜子保暖。"),
    ew("skirt", "/skɜːt/", "短裙", "Her skirt is red.", "她的短裙是红色的。"),
    lw("sweater", "/ˈswetə/", "毛衣", "The sweater is soft.", "这件毛衣很柔软。"),
    lw("scarf", "/skɑːf/", "围巾", "I wear a scarf in winter.", "我冬天戴围巾。"),
    // 家与家居
    ew("bed", "/bed/", "床", "I go to bed at nine.", "我九点上床睡觉。"),
    ew("door", "/dɔː/", "门", "Please close the door.", "请关上门。"),
    ew("window", "/ˈwɪndəʊ/", "窗户", "Open the window, please.", "请打开窗户。"),
    ew("sofa", "/ˈsəʊfə/", "沙发", "The cat sleeps on the sofa.", "猫在沙发上睡觉。"),
    ew("lamp", "/læmp/", "台灯", "I read under the lamp.", "我在台灯下看书。"),
    sw("mirror", "/ˈmɪrə/", "镜子", "I look at myself in the mirror.", "我照镜子看自己。"),
    lw("kitchen", "/ˈkɪtʃɪn/", "厨房", "Mom is cooking in the kitchen.", "妈妈在厨房做饭。"),
    sw("bathroom", "/ˈbɑːθruːm/", "浴室", "I wash my face in the bathroom.", "我在浴室洗脸。"),
    lw("garden", "/ˈɡɑːdn/", "花园", "We play in the garden.", "我们在花园里玩。"),
    // 动作
    ew("run", "/rʌn/", "跑", "I run to school every morning.", "我每天早上跑着去上学。"),
    ew("walk", "/wɔːk/", "走", "We walk to the park.", "我们步行去公园。"),
    ew("jump", "/dʒʌmp/", "跳", "The frog can jump high.", "青蛙能跳得很高。"),
    ew("swim", "/swɪm/", "游泳", "I swim in the pool.", "我在泳池里游泳。"),
    ew("sing", "/sɪŋ/", "唱歌", "She sings a sweet song.", "她唱一首好听的歌。"),
    ew("dance", "/dɑːns/", "跳舞", "They dance at the party.", "他们在聚会上跳舞。"),
    ew("read", "/riːd/", "阅读", "I read a story before bed.", "我睡前读一个故事。"),
    ew("write", "/raɪt/", "写", "I write a letter to Grandma.", "我给奶奶写信。"),
    ew("draw", "/drɔː/", "画画", "I draw a cat on paper.", "我在纸上画一只猫。"),
    ew("play", "/pleɪ/", "玩；打球", "We play games after class.", "我们课后玩游戏。"),
    ew("eat", "/iːt/", "吃", "We eat lunch at noon.", "我们中午吃午饭。"),
    ew("drink", "/drɪŋk/", "喝", "I drink water after running.", "我跑步后喝水。"),
    ew("sleep", "/sliːp/", "睡觉", "I sleep eight hours a night.", "我每晚睡八小时。"),
    ew("ride", "/raɪd/", "骑", "I ride a bike to school.", "我骑自行车上学。"),
    ew("buy", "/baɪ/", "买", "Mom buys fruit at the market.", "妈妈在市场买水果。"),
    ew("open", "/ˈəʊpən/", "打开", "Open your books, please.", "请打开书。"),
    ew("close", "/kləʊz/", "关闭", "Close the window, please.", "请关上窗户。"),
    ew("listen", "/ˈlɪsən/", "听", "Listen to the teacher, please.", "请听老师讲。"),
    ew("speak", "/spiːk/", "说", "Can you speak English?", "你会说英语吗？"),
    ew("watch", "/wɒtʃ/", "看", "I watch cartoons on TV.", "我在电视上看卡通。"),
    ew("look", "/lʊk/", "看", "Look at the blackboard.", "看黑板。"),
    ew("want", "/wɒnt/", "想要", "I want a new bike.", "我想要一辆新自行车。"),
    ew("like", "/laɪk/", "喜欢", "I like ice cream.", "我喜欢冰淇淋。"),
    ew("love", "/lʌv/", "爱", "I love my family.", "我爱我的家人。"),
    ew("learn", "/lɜːn/", "学习", "We learn new words today.", "我们今天学新单词。"),
    ew("teach", "/tiːtʃ/", "教", "She teaches us math.", "她教我们数学。"),
    // 形容词
    ew("big", "/bɪɡ/", "大的", "The elephant is big.", "大象很大。"),
    ew("small", "/smɔːl/", "小的", "The ant is small.", "蚂蚁很小。"),
    ew("tall", "/tɔːl/", "高的", "The tree is tall.", "这棵树很高。"),
    ew("short", "/ʃɔːt/", "矮的；短的", "My hair is short.", "我的头发很短。"),
    ew("long", "/lɒŋ/", "长的", "The river is long.", "这条河很长。"),
    ew("hot", "/hɒt/", "热的", "The soup is hot.", "汤很烫。"),
    ew("cold", "/kəʊld/", "冷的", "The water is cold.", "水很冷。"),
    ew("warm", "/wɔːm/", "温暖的", "Spring is warm.", "春天很温暖。"),
    ew("cool", "/kuːl/", "凉爽的", "The evening is cool.", "夜晚很凉爽。"),
    ew("happy", "/ˈhæpi/", "快乐的", "I am happy today.", "我今天很快乐。"),
    ew("sad", "/sæd/", "难过的", "Don't be sad.", "不要难过。"),
    ew("good", "/ɡʊd/", "好的", "This is a good book.", "这是一本好书。"),
    ew("bad", "/bæd/", "坏的", "The weather is bad today.", "今天天气不好。"),
    ew("new", "/njuː/", "新的", "I have a new schoolbag.", "我有一个新书包。"),
    ew("old", "/əʊld/", "旧的；老的", "This is an old house.", "这是一座旧房子。"),
    ew("fast", "/fɑːst/", "快的", "The car is very fast.", "这辆车很快。"),
    ew("slow", "/sləʊ/", "慢的", "The turtle is slow.", "乌龟很慢。"),
    ew("easy", "/ˈiːzi/", "容易的", "This question is easy.", "这个问题很简单。"),
    ew("hard", "/hɑːd/", "困难的；硬的", "The homework is hard.", "作业很难。"),
    ew("dirty", "/ˈdɜːti/", "脏的", "My hands are dirty.", "我的手很脏。"),
    lw("beautiful", "/ˈbjuːtɪfəl/", "美丽的", "The flowers are beautiful.", "这些花很美丽。"),
    ew("kind", "/kaɪnd/", "和蔼的", "Our teacher is kind.", "我们的老师很和蔼。"),
    ew("strong", "/strɒŋ/", "强壮的", "My father is strong.", "我父亲很强壮。"),
    ew("busy", "/ˈbɪzi/", "忙的", "Mom is busy cooking.", "妈妈正忙着做饭。"),
    ew("early", "/ˈɜːli/", "早的", "I get up early.", "我起得很早。"),
    ew("late", "/leɪt/", "晚的", "Don't be late for school.", "上学不要迟到。"),
    // 其他常用
    ew("name", "/neɪm/", "名字", "My name is Lily.", "我的名字是莉莉。"),
    ew("age", "/eɪdʒ/", "年龄", "What is your age?", "你多大了？"),
    ew("map", "/mæp/", "地图", "There is a map on the wall.", "墙上有一张地图。"),
    ew("key", "/kiː/", "钥匙", "I lost my key.", "我把钥匙丢了。"),
    ew("phone", "/fəʊn/", "电话", "I call my friend on the phone.", "我给朋友打电话。"),
    ew("bike", "/baɪk/", "自行车", "My bike is red.", "我的自行车是红色的。"),
    ew("bus", "/bʌs/", "公交车", "I take the bus to school.", "我坐公交车上学。"),
    ew("car", "/kɑː/", "汽车", "Dad drives the car to work.", "爸爸开车上班。"),
    ew("train", "/treɪn/", "火车", "The train is very long.", "火车很长。"),
    ew("plane", "/pleɪn/", "飞机", "The plane flies in the sky.", "飞机在天上飞。"),
    ew("park", "/pɑːk/", "公园", "We have a picnic in the park.", "我们在公园野餐。"),
    ew("shop", "/ʃɒp/", "商店", "The shop sells toys.", "这家商店卖玩具。"),
    ew("money", "/ˈmʌni/", "钱", "I save my money.", "我攒钱。"),
    ew("gift", "/ɡɪft/", "礼物", "The gift is for you.", "这份礼物是给你的。"),
    ew("ball", "/bɔːl/", "球", "I play with a ball.", "我玩球。"),
    lw("story", "/ˈstɔːri/", "故事", "Grandma tells me a story.", "奶奶给我讲故事。"),
    ew("song", "/sɒŋ/", "歌曲", "This song is my favorite.", "这首歌是我最喜欢的。"),
    ew("game", "/ɡeɪm/", "游戏", "We play a game together.", "我们一起玩游戏。"),
    lw("sport", "/spɔːt/", "运动", "Football is my favorite sport.", "足球是我最喜欢的运动。"),
];

const JUNIOR_WORDS: &[SeedWord] = &[
    w("believe", "/bɪˈliːv/", "相信", "I believe you can do it.", "我相信你能做到。"),
    lw("borrow", "/ˈbɒrəʊ/", "借入", "Can I borrow your dictionary?", "我可以借你的字典吗？"),
    sw("century", "/ˈsentʃəri/", "世纪", "The museum was built in the 19th century.", "这座博物馆建于 19 世纪。"),
    sw("dictionary", "/ˈdɪkʃənəri/", "词典", "Look up the word in the dictionary.", "在词典里查一下这个词。"),
    sw("environment", "/ɪnˈvaɪrənmənt/", "环境", "We should protect the environment.", "我们应该保护环境。"),
    lw("favorite", "/ˈfeɪvərɪt/", "最喜爱的", "Swimming is my favorite sport.", "游泳是我最喜爱的运动。"),
    sw("geography", "/dʒiˈɒɡrəfi/", "地理", "Geography helps us understand the world.", "地理帮助我们了解世界。"),
    lw("hospital", "/ˈhɒspɪtl/", "医院", "She works at the city hospital.", "她在市医院工作。"),
    lw("invite", "/ɪnˈvaɪt/", "邀请", "Let's invite Tom to the party.", "我们邀请汤姆来参加聚会吧。"),
    sw("journey", "/ˈdʒɜːni/", "旅程", "The journey took three hours by train.", "这段旅程坐火车花了三个小时。"),
    sw("knowledge", "/ˈnɒlɪdʒ/", "知识", "Knowledge is power.", "知识就是力量。"),
    lw("library", "/ˈlaɪbrəri/", "图书馆", "I study in the library after class.", "课后我在图书馆学习。"),
    w("memory", "/ˈmeməri/", "记忆；记忆力", "Exercise can improve your memory.", "锻炼能提高记忆力。"),
    sw("neighbor", "/ˈneɪbə/", "邻居", "Our neighbor is very friendly.", "我们的邻居很友好。"),
    sw("opportunity", "/ˌɒpəˈtjuːnəti/", "机会", "Don't miss this good opportunity.", "不要错过这个好机会。"),
    lw("practice", "/ˈpræktɪs/", "练习", "Practice makes perfect.", "熟能生巧。"),
    lw("quiet", "/ˈkwaɪət/", "安静的", "Please keep quiet in the library.", "在图书馆请保持安静。"),
    w("respect", "/rɪˈspekt/", "尊重", "We should respect different opinions.", "我们应该尊重不同的意见。"),
    w("science", "/ˈsaɪəns/", "科学", "Science explains how things work.", "科学解释事物如何运作。"),
    lw("travel", "/ˈtrævl/", "旅行", "I want to travel around the world.", "我想环游世界。"),
    lw("useful", "/ˈjuːsfəl/", "有用的", "This map is very useful for tourists.", "这张地图对游客很有用。"),
    lw("volunteer", "/ˌvɒlənˈtɪə/", "志愿者；自愿做", "Many students volunteer at the shelter.", "许多学生在收容所做志愿者。"),
    lw("weather", "/ˈweðə/", "天气", "The weather is sunny today.", "今天天气晴朗。"),
    lw("yesterday", "/ˈjestədeɪ/", "昨天", "Yesterday we visited the science museum.", "昨天我们参观了科学博物馆。"),
    w("advice", "/ədˈvaɪs/", "建议", "Thank you for your useful advice.", "谢谢你有用的建议。"),
    w("experience", "/ɪkˈspɪəriəns/", "经验；经历", "She has rich teaching experience.", "她有丰富的教学经验。"),
    lw("habit", "/ˈhæbɪt/", "习惯", "Reading is a good habit.", "阅读是一个好习惯。"),
    w("success", "/səkˈses/", "成功", "Hard work leads to success.", "努力带来成功。"),
];

const SENIOR_WORDS: &[SeedWord] = &[
    sw("abundant", "/əˈbʌndənt/", "丰富的；充足的", "The region has abundant natural resources.", "该地区自然资源丰富。"),
    sw("circumstance", "/ˈsɜːkəmstəns/", "情况；环境", "Under these circumstances, we have to wait.", "在这种情况下，我们只能等待。"),
    hw("dilemma", "/dɪˈlemə/", "困境；进退两难", "She faced a dilemma between work and family.", "她在工作和家庭之间进退两难。"),
    w("endeavor", "/ɪnˈdevə/", "努力", "We must endeavor to improve our skills.", "我们必须努力提升技能。"),
    w("genuine", "/ˈdʒenjuɪn/", "真诚的；真正的", "He showed genuine interest in the project.", "他对这个项目表现出真诚的兴趣。"),
    nw("hierarchy", "/ˈhaɪərɑːki/", "等级制度", "The company has a clear hierarchy.", "这家公司有清晰的等级制度。"),
    sw("inevitable", "/ɪnˈevɪtəbl/", "不可避免的", "Change is inevitable in any growing city.", "任何成长中的城市都不可避免变化。"),
    sw("legitimate", "/lɪˈdʒɪtɪmət/", "合法的；正当的", "Is that a legitimate reason for leaving early?", "那是提前离开的正当理由吗？"),
    sw("momentum", "/məˈmentəm/", "势头；动量", "The campaign is gaining momentum.", "这场运动势头正盛。"),
    w("negotiate", "/nɪˈɡəʊʃieɪt/", "谈判；协商", "We need to negotiate a better deadline.", "我们需要协商一个更好的截止时间。"),
    sw("obscure", "/əbˈskjʊə/", "模糊的；鲜为人知的", "The meaning of this poem is obscure.", "这首诗的含义很晦涩。"),
    nw("paramount", "/ˈpærəmaʊnt/", "至为重要的", "Safety is paramount in this industry.", "在这个行业，安全至关重要。"),
    w("quantify", "/ˈkwɒntɪfaɪ/", "量化", "It is hard to quantify the value of health.", "健康的价值很难量化。"),
    sw("resilient", "/rɪˈzɪliənt/", "有韧性的", "Children are often more resilient than adults expect.", "孩子往往比大人想象中更有韧性。"),
    sw("sophisticated", "/səˈfɪstɪkeɪtɪd/", "复杂的；精密的", "This is a sophisticated piece of software.", "这是一个精密的软件。"),
    w("transparent", "/trænsˈpærənt/", "透明的", "The process should be transparent and fair.", "流程应当透明公平。"),
    sw("unanimous", "/juˈnænɪməs/", "一致同意的", "The board reached a unanimous decision.", "董事会达成了一致决定。"),
    w("viable", "/ˈvaɪəbl/", "可行的；可存活的", "This plan is not economically viable.", "这个计划在经济上不可行。"),
    sw("withstand", "/wɪðˈstænd/", "承受；抵挡", "The bridge can withstand strong winds.", "这座桥能承受强风。"),
    w("yield", "/jiːld/", "产生；屈服", "The research yielded surprising results.", "研究产生了令人惊讶的结果。"),
    hw("zenith", "/ˈzenɪθ/", "顶峰；最高点", "The empire was at its zenith in the 15th century.", "该帝国在 15 世纪达到鼎盛。"),
    w("feasible", "/ˈfiːzəbl/", "可行的", "A full redesign is not feasible this quarter.", "本季度全面改版不可行。"),
    w("sustain", "/səˈsteɪn/", "维持；支撑", "Small habits can sustain long-term progress.", "小习惯可以维持长期进步。"),
    w("acknowledge", "/əkˈnɒlɪdʒ/", "承认；致谢", "She acknowledged the effort behind the report.", "她认可了这份报告背后的付出。"),
    w("concise", "/kənˈsaɪs/", "简洁的", "Keep your summary concise and clear.", "让你的总结简洁清晰。"),
    w("generate", "/ˈdʒenəreɪt/", "产生；生成", "This exercise will generate new ideas.", "这个练习会产生新想法。"),
    nw("perspective", "/pəˈspektɪv/", "视角；观点", "Travel gives you a new perspective on life.", "旅行会给你看待生活的新视角。"),
    w("subtle", "/ˈsʌtl/", "微妙的；不易察觉的", "There is a subtle difference between the two words.", "这两个词之间有微妙的差别。"),
];

const COLLEGE_WORDS: &[SeedWord] = &[
    w("abstract", "/ˈæbstrækt/", "抽象的", "The theory is too abstract for beginners.", "这个理论对初学者来说太抽象了。"),
    sw("ambiguous", "/æmˈbɪɡjuəs/", "模棱两可的", "The ending is deliberately ambiguous.", "这个结尾故意留白。"),
    sw("coherent", "/kəʊˈhɪərənt/", "连贯的", "The essay needs a more coherent structure.", "这篇文章需要更连贯的结构。"),
    sw("criterion", "/kraɪˈtɪəriən/", "标准", "Price is not the only criterion.", "价格不是唯一的标准。"),
    w("empirical", "/ɪmˈpɪrɪkl/", "经验主义的；实证的", "The theory needs empirical support.", "这个理论需要实证支持。"),
    sw("framework", "/ˈfreɪmwɜːk/", "框架", "We built a framework for future research.", "我们为未来研究搭建了框架。"),
    hw("heuristic", "/hjuˈrɪstɪk/", "启发式的", "The heuristic method saves time.", "启发式方法节省时间。"),
    dw("ideology", "/ˌaɪdiˈɒlədʒi/", "意识形态", "Ideology shapes the way we think.", "意识形态塑造我们的思维方式。"),
    tw("paradigm", "/ˈpærədaɪm/", "范式", "The discovery created a new scientific paradigm.", "这一发现开创了新的科学范式。"),
    w("pragmatic", "/præɡˈmætɪk/", "务实的", "We need a pragmatic approach to this problem.", "我们需要务实地处理这个问题。"),
    w("rigorous", "/ˈrɪɡərəs/", "严格的；严谨的", "The experiment followed rigorous standards.", "实验遵循严格的标准。"),
    hw("semantics", "/sɪˈmæntɪks/", "语义学", "The semantics of this term are debated.", "这个术语的语义有争议。"),
    dw("taxonomy", "/tækˈsɒnəmi/", "分类学", "Biologists use a taxonomy to classify species.", "生物学家用分类法给物种分类。"),
    sw("utilitarian", "/ˌjuːtɪlɪˈteəriən/", "功利主义的；实用的", "The design is purely utilitarian.", "这个设计纯粹是实用主义的。"),
    w("verify", "/ˈverɪfaɪ/", "核实", "Please verify the data before publishing.", "发表前请核实数据。"),
    w("threshold", "/ˈθreʃhəʊld/", "门槛；临界点", "We are approaching the threshold of a new era.", "我们正接近一个新时代的门槛。"),
    w("deliberate", "/dɪˈlɪbərət/", "深思熟虑的；刻意的", "She made a deliberate effort to speak slowly.", "她刻意放慢语速。"),
    sw("nuance", "/ˈnjuːɑːns/", "细微差别", "The writer captures every nuance of the dialogue.", "作者捕捉到了对话中的每个细微差别。"),
    sw("empathy", "/ˈempəθi/", "共情；同理心", "Good listeners show empathy without judgment.", "好的倾听者不带评判地共情。"),
    sw("hypothesis", "/haɪˈpɒθəsɪs/", "假说；假设", "The data supports our initial hypothesis.", "数据支持我们最初的假设。"),
    sw("cognitive", "/ˈkɒɡnətɪv/", "认知的", "Reading improves cognitive ability.", "阅读提升认知能力。"),
    w("consensus", "/kənˈsensəs/", "共识", "The team reached a consensus quickly.", "团队很快达成了共识。"),
    w("diminish", "/dɪˈmɪnɪʃ/", "减少；减弱", "His influence began to diminish.", "他的影响力开始减弱。"),
    w("explicit", "/ɪkˈsplɪsɪt/", "明确的", "The instructions were explicit and easy to follow.", "说明明确且易于遵循。"),
    w("incentive", "/ɪnˈsentɪv/", "激励", "Bonus is a strong incentive for staff.", "奖金对员工是强大的激励。"),
    w("magnitude", "/ˈmæɡnɪtjuːd/", "巨大；量级", "We did not realize the magnitude of the problem.", "我们没有意识到问题的严重性。"),
];

const IELTS_WORDS: &[SeedWord] = &[
    w("alleviate", "/əˈliːvieɪt/", "缓解；减轻", "The medicine helps alleviate the pain.", "这种药有助于缓解疼痛。"),
    nw("bureaucracy", "/bjʊəˈrɒkrəsi/", "官僚机构", "Too much bureaucracy slows down decisions.", "过多的官僚程序拖慢了决策。"),
    sw("detrimental", "/ˌdetrɪˈmentl/", "有害的", "Smoking is detrimental to health.", "吸烟有害健康。"),
    sw("exacerbate", "/ɪɡˈzæsəbeɪt/", "使恶化", "Stress can exacerbate the symptoms.", "压力会加剧症状。"),
    w("facilitate", "/fəˈsɪlɪteɪt/", "促进", "Technology facilitates global communication.", "技术促进全球交流。"),
    sw("globalization", "/ˌɡləʊbəlaɪˈzeɪʃn/", "全球化", "Globalization has changed local economies.", "全球化改变了地方经济。"),
    nw("homogeneous", "/ˌhɒməˈdʒiːniəs/", "同质的", "The population is fairly homogeneous.", "人口构成相当同质。"),
    sw("infrastructure", "/ˈɪnfrəstrʌktʃə/", "基础设施", "The country invested heavily in infrastructure.", "该国大力投资基础设施。"),
    hw("juxtapose", "/ˌdʒʌkstəˈpəʊz/", "并列", "The essay juxtaposes two different viewpoints.", "文章并列了两种不同观点。"),
    w("mitigate", "/ˈmɪtɪɡeɪt/", "缓解；减轻", "We need measures to mitigate climate change.", "我们需要缓解气候变化的措施。"),
    sw("phenomenon", "/fəˈnɒmɪnən/", "现象", "Urban migration is a global phenomenon.", "城市迁移是一种全球现象。"),
    w("prevalent", "/ˈprevələnt/", "普遍的", "This belief is prevalent in many cultures.", "这种信念在许多文化中都很普遍。"),
    hw("scrutinize", "/ˈskruːtənaɪz/", "仔细审查", "The report was scrutinized by experts.", "这份报告经过了专家的仔细审查。"),
    sw("sustainability", "/səˌsteɪnəˈbɪləti/", "可持续性", "Sustainability is key to urban planning.", "可持续性是城市规划的关键。"),
    w("undermine", "/ˌʌndəˈmaɪn/", "削弱", "Late nights can undermine your health.", "熬夜会损害健康。"),
    w("advocate", "/ˈædvəkeɪt/", "提倡；拥护", "Many experts advocate a plant-based diet.", "许多专家提倡以植物为主的饮食。"),
    w("controversial", "/ˌkɒntrəˈvɜːʃl/", "有争议的", "The new policy is highly controversial.", "新政策争议很大。"),
    w("demographic", "/ˌdeməˈɡræfɪk/", "人口统计的", "The demographic shift affects the labor market.", "人口结构变化影响劳动力市场。"),
    w("fluctuate", "/ˈflʌktʃueɪt/", "波动", "Prices fluctuate with supply and demand.", "价格随供需波动。"),
    w("plausible", "/ˈplɔːzəbl/", "貌似合理的", "Her explanation sounds plausible.", "她的解释听起来有道理。"),
    tw("serendipity", "/ˌserənˈdɪpəti/", "意外发现美好事物的能力", "Finding that old letter was pure serendipity.", "找到那封旧信完全是意外之喜。"),
    w("articulate", "/ɑːˈtɪkjuleɪt/", "清晰表达（的）", "He is articulate when explaining complex ideas.", "解释复杂想法时他表达得很清楚。"),
    w("coherent", "/kəʊˈhɪərənt/", "连贯的；有条理的", "The essay needs a more coherent structure.", "这篇文章需要更连贯的结构。"),
    w("viable", "/ˈvaɪəbl/", "可行的", "This plan is not economically viable.", "这个计划在经济上不可行。"),
    w("cognitive", "/ˈkɒɡnətɪv/", "认知的", "Reading improves cognitive ability.", "阅读提升认知能力。"),
    w("inevitable", "/ɪnˈevɪtəbl/", "不可避免的", "Change is inevitable in any growing city.", "任何成长中的城市都不可避免变化。"),
    tw("paradigm", "/ˈpærədaɪm/", "范式", "The discovery created a new scientific paradigm.", "这一发现开创了新的科学范式。"),
    w("incentive", "/ɪnˈsentɪv/", "激励", "Bonus is a strong incentive for staff.", "奖金对员工是强大的激励。"),
];

const PROFESSIONAL_WORDS: &[SeedWord] = &[
    w("algorithm", "/ˈælɡərɪðəm/", "算法", "The algorithm sorts data in milliseconds.", "该算法在毫秒内完成数据排序。"),
    w("bandwidth", "/ˈbændwɪdθ/", "带宽", "The video needs high bandwidth.", "该视频需要高带宽。"),
    sw("capacitor", "/kəˈpæsɪtə/", "电容器", "The capacitor stores electrical energy.", "电容器储存电能。"),
    nw("depreciation", "/dɪˌpriːʃiˈeɪʃn/", "折旧", "Asset depreciation affects the balance sheet.", "资产折旧影响资产负债表。"),
    sw("entropy", "/ˈentrəpi/", "熵", "Entropy measures disorder in a system.", "熵衡量系统的无序程度。"),
    w("fiscal", "/ˈfɪskl/", "财政的", "The government adopted a tight fiscal policy.", "政府采取了紧缩的财政政策。"),
    sw("genome", "/ˈdʒiːnəʊm/", "基因组", "Scientists mapped the human genome.", "科学家绘制了人类基因组图谱。"),
    vw("hemodynamics", "/ˌhiːmədaɪˈnæmɪks/", "血流动力学", "Hemodynamics studies blood flow in vessels.", "血流动力学研究血管中的血液流动。"),
    sw("jurisdiction", "/ˌdʒʊərɪsˈdɪkʃn/", "管辖权", "The court has jurisdiction over this case.", "法院对此案有管辖权。"),
    w("kinetic", "/kɪˈnetɪk/", "动能的", "Kinetic energy increases with speed.", "动能随速度增加而增加。"),
    sw("liquidity", "/lɪˈkwɪdəti/", "流动性", "The bank maintains high liquidity.", "银行保持高流动性。"),
    w("molecule", "/ˈmɒlɪkjuːl/", "分子", "A water molecule contains two hydrogen atoms.", "一个水分子包含两个氢原子。"),
    sw("neurology", "/njʊˈrɒlədʒi/", "神经学", "Neurology studies the nervous system.", "神经学研究神经系统。"),
    w("protocol", "/ˈprəʊtəkɒl/", "协议", "The devices communicate using a secure protocol.", "设备使用安全协议通信。"),
    w("quantum", "/ˈkwɒntəm/", "量子的", "Quantum computing is advancing rapidly.", "量子计算正在快速发展。"),
    w("revenue", "/ˈrevənjuː/", "收入", "Online sales drive most of the revenue.", "在线销售贡献了大部分收入。"),
    hw("semiconductor", "/ˌsemikənˈdʌktə/", "半导体", "Silicon is the main semiconductor material.", "硅是主要的半导体材料。"),
    vw("thermodynamics", "/ˌθɜːmədaɪˈnæmɪks/", "热力学", "Thermodynamics explains how engines convert heat.", "热力学解释发动机如何转换热量。"),
    w("venture", "/ˈventʃə/", "风险项目；冒险", "They launched a new tech venture.", "他们启动了一个新的科技项目。"),
    sw("volatility", "/ˌvɒləˈtɪləti/", "波动性", "Market volatility scares small investors.", "市场波动吓坏了小投资者。"),
    w("derivative", "/dɪˈrɪvətɪv/", "导数；衍生品", "The derivative measures how a function changes.", "导数衡量函数的变化率。"),
    w("encryption", "/ɪnˈkrɪpʃn/", "加密", "Encryption protects sensitive data.", "加密保护敏感数据。"),
    sw("forensic", "/fəˈrensɪk/", "法医的", "Forensic science helps solve crimes.", "法医科学帮助破案。"),
    w("leverage", "/ˈliːvərɪdʒ/", "杠杆；利用", "The firm uses leverage to expand.", "该公司利用杠杆扩张。"),
    w("metabolism", "/məˈtæbəlɪzəm/", "新陈代谢", "Exercise speeds up your metabolism.", "运动加快新陈代谢。"),
    w("portfolio", "/pɔːtˈfəʊliəʊ/", "投资组合", "A diversified portfolio reduces risk.", "多元化的投资组合降低风险。"),
];

pub const BOOKS: &[SeedBook] = &[
    SeedBook {
        code: "primary",
        title: "小学英语",
        description: "常见生活词汇，适合零基础到小学阶段。",
        level: "a1",
        difficulty: 0.7,
        sort_order: 1,
        words: PRIMARY_WORDS,
    },
    SeedBook {
        code: "junior",
        title: "初中英语",
        description: "中考核心词汇与常用表达。",
        level: "a2",
        difficulty: 0.9,
        sort_order: 2,
        words: JUNIOR_WORDS,
    },
    SeedBook {
        code: "senior",
        title: "高中英语",
        description: "高考高频词汇与进阶用法。",
        level: "b1",
        difficulty: 1.1,
        sort_order: 3,
        words: SENIOR_WORDS,
    },
    SeedBook {
        code: "college",
        title: "大学英语",
        description: "大学英语四六级核心词汇。",
        level: "b2",
        difficulty: 1.3,
        sort_order: 4,
        words: COLLEGE_WORDS,
    },
    SeedBook {
        code: "ielts",
        title: "雅思",
        description: "雅思考试高频学术词汇。",
        level: "c1",
        difficulty: 1.5,
        sort_order: 5,
        words: IELTS_WORDS,
    },
    SeedBook {
        code: "professional",
        title: "专业英语",
        description: "计算机、金融、医学等专业术语。",
        level: "c1",
        difficulty: 1.7,
        sort_order: 6,
        words: PROFESSIONAL_WORDS,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_books_cover_six_levels() {
        assert_eq!(BOOKS.len(), 6);
        let codes: Vec<&str> = BOOKS.iter().map(|book| book.code).collect();
        assert!(codes.contains(&"primary"));
        assert!(codes.contains(&"ielts"));
        assert!(codes.contains(&"professional"));
    }

    #[test]
    fn every_book_has_words_and_phonetics() {
        for book in BOOKS {
            assert!(!book.words.is_empty(), "{} 词书不应为空", book.code);
            assert!(book.difficulty >= 0.5 && book.difficulty <= 2.0);
            for seed in book.words {
                assert!(!seed.word.is_empty());
                assert!(!seed.phonetic.is_empty());
                assert!(!seed.meaning.is_empty());
                assert!(seed.difficulty >= 0.5 && seed.difficulty <= 2.0);
            }
        }
    }

    #[test]
    fn difficulty_grows_with_stage() {
        let difficulties: Vec<f64> = BOOKS.iter().map(|book| book.difficulty).collect();
        let mut sorted = difficulties.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(difficulties, sorted);
        assert!(difficulties[0] < difficulties[5]);
    }

    #[test]
    fn primary_book_has_substantial_words() {
        assert!(
            PRIMARY_WORDS.len() >= 150,
            "小学词库应包含 150 个以上单词，当前 {} 个",
            PRIMARY_WORDS.len()
        );
    }

}
